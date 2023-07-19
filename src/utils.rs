use std::io::Result;
use image::{ImageFormat, RgbaImage};
use image::error::{ImageResult, ImageError};
use md5::{Digest, Md5};
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;

async fn is_image<P: AsRef<Path>>(path: P) -> Result<bool> {
    let mut file = File::open(path).await?;
    let mut buff = [0; 4];
    file.read_exact(&mut buff).await?;
    Ok(infer::is_image(&buff))
}

fn hash<P: AsRef<Path>>(path: P) -> String {
    let mut hasher = Md5::new();
    hasher.update(path.as_ref().to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    format!("{:x}.png", digest)
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
    if paths.is_empty() {
        Err(Error::new(ErrorKind::InvalidInput, "no valid image found"))
    } else {
        Ok(paths)
    }
}

pub fn thumb_path<P: AsRef<Path>>(image_path: P) -> PathBuf {
    let cache_dir = dirs::cache_dir().unwrap().join("fuu");
    let hashed_path = hash(&image_path);
    cache_dir.join(hashed_path)
}

pub async fn create_cache_dir() -> Result<()> {
    let cache_dir = dirs::cache_dir().unwrap().join("fuu");
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir).await?;
    }
    Ok(())
}

pub async fn image_open<P: AsRef<Path>>(image_path: P) -> ImageResult<RgbaImage> {
    let input_file = File::open(&image_path).await.map_err(ImageError::IoError)?;
    let reader = BufReader::new(input_file.into_std().await);

    Ok(image::load(reader, ImageFormat::from_path(&image_path)?)?.into_rgba8())
}

pub async fn generate_thumb<P: AsRef<Path>>(image_path: P, dest: P, new_dim: (u32, u32)) -> ImageResult<()> {
    let input_file = File::open(&image_path).await.map_err(ImageError::IoError)?;
    let reader = BufReader::new(input_file.into_std().await);

    let input_image = image::load(reader, ImageFormat::from_path(&image_path)?)?;

    let output_image = input_image.thumbnail(new_dim.0, new_dim.1);

    let mut output_file = File::create(dest).await.map_err(ImageError::IoError)?.into_std().await;
    output_image.write_to(&mut output_file, image::ImageFormat::Png)
}
