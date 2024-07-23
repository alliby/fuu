use hashbrown::HashMap;
use image::imageops::FilterType;
use image::RgbaImage;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use vello::peniko::{Blob, Format, Image};

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

type StoreMap = HashMap<PathBuf, ImageCacheState>;

const THUMB_DIM: u32 = 300;
const MAX_IMAGE_DIM: u32 = 4000;

const CACHE_SIZE_LIMIT: usize = 500 * 1024 * 1024; // 500MB
const IMAGE_SIZE_LIMIT: u32 = 4 * MAX_IMAGE_DIM * MAX_IMAGE_DIM;

static CACHE_SIZE: AtomicUsize = AtomicUsize::new(0);
static IMAGE_STORE: Lazy<Mutex<StoreMap>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Default)]
pub enum ImageCacheState {
    #[default]
    FirstAppend,
    Requested,
    Ready {
        preview: RgbaImage,
        thumb: RgbaImage,
    },
}

#[derive(Clone, Default)]
pub enum ImageState {
    #[default]
    Loading,
    Loaded(Image),
}

#[derive(Clone, Copy)]
pub enum ImageSize {
    Thumbnail,
    Preview,
}

fn store_image(path: PathBuf) -> anyhow::Result<()> {
    // read and decode Image
    let data = std::fs::read(&path)?;
    let mut image = image::io::Reader::new(std::io::Cursor::new(data))
        .with_guessed_format()?
        .decode()?;
    let width = image.width();
    let height = image.height();
    if 4 * width * height > IMAGE_SIZE_LIMIT {
        image = image.resize(MAX_IMAGE_DIM, MAX_IMAGE_DIM, FilterType::Triangle);
    }
    // creating the thumbnail
    let thumb = image.thumbnail(THUMB_DIM, THUMB_DIM);
    // Chech and clear the Cache
    let image_size = (width * height * 4) as usize;
    let store_size = CACHE_SIZE.fetch_add(image_size, Ordering::SeqCst);
    let mut image_store = IMAGE_STORE.lock();
    if store_size >= CACHE_SIZE_LIMIT {
        image_store.clear();
    }
    // insert the images to the store
    image_store.insert(
        path,
        ImageCacheState::Ready {
            preview: image.into_rgba8(),
            thumb: thumb.into_rgba8(),
        },
    );
    Ok(())
}

pub fn get_image(path: impl AsRef<Path>, size: ImageSize) -> ImageState {
    let path = path.as_ref();
    let mut store = IMAGE_STORE.lock();
    match store.get(path) {
        Some(ImageCacheState::FirstAppend) => {
            let path = path.to_owned();
            store.insert(path.clone(), ImageCacheState::Requested);
            drop(store);
            rayon::spawn(move || store_image(path).unwrap());
            ImageState::Loading
        }
        Some(ImageCacheState::Requested) => ImageState::Loading,
        Some(ImageCacheState::Ready { preview, thumb }) => {
            let rgba = match size {
                ImageSize::Thumbnail => thumb,
                ImageSize::Preview => preview,
            };
            let data = Arc::new(rgba.as_raw().clone());
            let blob = Blob::new(data);
            let image = Image::new(blob, Format::Rgba8, rgba.width(), rgba.height());
            ImageState::Loaded(image)
        }
        None => {
            store.insert(path.to_owned(), ImageCacheState::FirstAppend);
            ImageState::Loading
        }
    }
}
