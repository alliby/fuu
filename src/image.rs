use hashbrown::HashMap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use vello::peniko::{Blob, Format, Image};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

type StoreMap = HashMap<PathBuf, ImageCacheState>;

const CACHE_SIZE_LIMIT: usize = 100 * 1024 * 1024; // 100MB
static CACHE_SIZE: AtomicUsize = AtomicUsize::new(0);
static IMAGE_STORE: Lazy<Mutex<StoreMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Default)]
enum ImageCacheState {
    #[default]
    Idle,
    Requested,
    Ready(Image),
}

#[derive(Clone, Default)]
pub enum ImageState {
    #[default]
    Loading,
    Loaded(Image),
}

fn decode_image(data: &[u8]) -> anyhow::Result<Image> {
    let image = image::io::Reader::new(std::io::Cursor::new(data))
        .with_guessed_format()?
        .decode()?;
    let image = image.thumbnail(1000, 1000);
    let width = image.width();
    let height = image.height();
    let data = Arc::new(image.into_rgba8().into_vec());
    let blob = Blob::new(data);
    Ok(Image::new(blob, Format::Rgba8, width, height))
}

pub fn get_image(path: impl AsRef<Path>) -> ImageState {
    let path = path.as_ref();
    let mut store = IMAGE_STORE.lock();
    match store.get(path) {
        Some(ImageCacheState::Idle) => {
            let path = path.to_owned();
            store.insert(path.clone(), ImageCacheState::Requested);
            drop(store);
            rayon::spawn(move || {
                let data = std::fs::read(&path).unwrap();
                let image = decode_image(&data).unwrap();
                let image_size = (image.width * image.height * 4) as usize;
                let store_size = CACHE_SIZE.fetch_add(image_size, Ordering::SeqCst);
                let mut store = IMAGE_STORE.lock();
                if store_size >= CACHE_SIZE_LIMIT {
                    store.clear();
                }
                store.insert(path, ImageCacheState::Ready(image));
            });
            ImageState::Loading
        }
        Some(ImageCacheState::Requested) => ImageState::Loading,
        Some(ImageCacheState::Ready(image)) => ImageState::Loaded(image.clone()),
        None => {
            store.insert(path.to_owned(), ImageCacheState::Idle);
            ImageState::Loading
        }
    }
}
