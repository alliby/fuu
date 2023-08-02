use std::os::unix::ffi::OsStrExt;
use crate::gui::types::*;
use image::error::{ImageError, ImageResult};
use md5::{Digest, Md5};
use std::io::BufReader;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use bytes::Bytes;

fn hash<S: AsRef<OsStr>>(file_name: S) -> String {
    let mut hasher = Md5::new();
    hasher.update(file_name.as_ref().as_bytes());
    let digest = hasher.finalize();
    format!("{:x}.png", digest)
}

pub fn thumb_path<S: AsRef<OsStr>>(file_name: S) -> PathBuf {
    let cache_dir = dirs::cache_dir().unwrap().join("fuu");
    let hashed_name = hash(file_name);
    cache_dir.join(hashed_name)
}

async fn is_image<P: AsRef<Path>>(path: P) -> Result<bool> {
    if !path.as_ref().is_file() {
        return Ok(false);
    }
    let mut file = File::open(path).await?;
    let mut buff = [0; 4];
    file.read_exact(&mut buff).await?;
    Ok(infer::is_image(&buff))
}

pub async fn read_dir<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let mut entries = fs::read_dir(path).await?;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_file() && is_image(&path).await.unwrap_or(false) {
            paths.push(path)
        }
    }
    Ok(paths)
}

pub async fn read_sources(sources: Vec<ImageSource>) -> Vec<ImageSource> {
    let mut output_sources = Vec::with_capacity(sources.len());
    for source in sources {
        if let ImageSource::Path(path) = source {
            if path.is_dir() {
                let children = read_dir(path).await.unwrap_or_default();
                output_sources.extend(children.into_iter().map(ImageSource::Path))
            } else if is_image(&path).await.unwrap_or(false) {
                output_sources.push(ImageSource::Path(path))
            }
        } else {
            output_sources.push(source)
        }
    }
    output_sources
}

pub async fn create_cache_dir() -> Result<()> {
    let cache_dir = dirs::cache_dir().unwrap().join("fuu");
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir).await?;
    }
    Ok(())
}

pub async fn image_dimensions<P: AsRef<Path>>(image_path: P) -> ImageResult<(u32, u32)> {
    let input_file = File::open(&image_path).await.map_err(ImageError::IoError)?;
    let reader = BufReader::new(input_file.into_std().await);
    image::io::Reader::new(reader)
        .with_guessed_format()?
        .into_dimensions()
}

// unfortunately we cannot construct new error from reqwest::Error
async fn fetch_url(url: url::Url) -> std::result::Result<Bytes, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::{Error, ErrorKind};
    let bytes = reqwest::get(url).await?.bytes().await?;
    if !infer::is_image(&bytes) {
        return Err(Box::new(Error::from(ErrorKind::InvalidData)));
    }
    Ok(bytes)
}

async fn fetch_file<P: AsRef<Path>>(file_path: P) -> Result<Bytes> {
    fs::read(file_path).await.map(Bytes::from)
}

pub async fn fetch_source(source: ImageSource) -> Option<Bytes> {
    match source {
        ImageSource::Path(path) => fetch_file(path).await.ok(),
        ImageSource::Url(url) => {
            let image_path = thumb_path(url.as_str());
            if image_path.exists() {
                fetch_file(image_path).await.ok()
            } else {
                fetch_url(url).await.ok()
            }
        }
    }
}

pub async fn generate_thumb(image_card: ImageCard) -> Option<(u32,u32)> {
    match (image_card.preview, image_card.thumb) {
        (ImageSource::Path(preview_path), ImageSource::Path(thumb_dest)) => {
            if preview_path.exists() && thumb_dest.exists() {
                return Some(image_dimensions(thumb_dest).await.ok()?)
            }
            let input_file = File::open(&preview_path).await.ok()?;
            let reader = BufReader::new(input_file.into_std().await);
            let input_image = image::io::Reader::new(reader)
                .with_guessed_format().ok()?
                .decode().ok()?;
            let ratio = input_image.width() as f32 / input_image.height() as f32;
            let new_height = (image_card.width as f32 / ratio) as u32;
            let mut writer = File::create(thumb_dest)
                .await.ok()?
                .into_std()
                .await;
            let thumb_image = input_image.thumbnail(image_card.width, image_card.height);
            thumb_image.write_to(&mut writer, image::ImageFormat::Png).ok()?;
            Some((image_card.width, new_height))
        }
        (ImageSource::Url(preview_url), ImageSource::Path(thumb_dest)) => {
            let preview_path = thumb_path(preview_url.as_str());
            if preview_path.exists() && thumb_dest.exists() {
                return Some(image_dimensions(thumb_dest).await.ok()?)
            }
            let preview_data = fetch_url(preview_url).await.ok()?;
            let input_image = image::load_from_memory(&preview_data).ok()?;
            let ratio = input_image.width() as f32 / input_image.height() as f32;
            let new_height = (image_card.width as f32 / ratio) as u32;
            if !thumb_dest.exists() {
                let mut thumb_writer = File::create(thumb_dest)
                    .await.ok()?
                    .into_std()
                    .await;
                let thumb_image = input_image.thumbnail(image_card.width, image_card.height);
                thumb_image.write_to(&mut thumb_writer, image::ImageFormat::Png).ok()?;
            }
            if !preview_path.exists() {
                fs::write(preview_path, preview_data).await.ok()?;
            }
            Some((image_card.width, new_height))
        }
        _ => None,
    }
}
