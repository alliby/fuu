use crate::gui::style::DEFAULT_IMG_WIDTH;
use crate::utils::{image_open, thumb_path};
use image::image_dimensions;
use std::path::PathBuf;

#[derive(Default)]
pub enum ThumbState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Default)]
pub enum ImageState {
    #[default]
    Loading,
    Loaded(ImageData),
    Error
}

#[derive(Default)]
pub struct ImageCard {
    pub image_path: PathBuf,
    pub thumb_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub thumb_state: ThumbState,
    pub image_state: ImageState,
}

impl ImageCard {
    pub fn resize(&self, new_width: u32) -> (u32, u32) {
        let ratio = self.width as f32 / self.height as f32;
        let new_height = (new_width as f32 / ratio) as u32;
        (new_width, new_height)
    }

    pub fn new(image_path: PathBuf) -> Self {
        let (width, height) = image_dimensions(&image_path).unwrap_or((DEFAULT_IMG_WIDTH, DEFAULT_IMG_WIDTH));
        let thumb_path = thumb_path(&image_path);
        let ratio = width as f32 / height as f32;
        let new_height = (DEFAULT_IMG_WIDTH as f32 / ratio) as u32;
        let thumb_state = if thumb_path.exists() {
            ThumbState::Loaded
        } else {
            ThumbState::Loading
        };
        Self {
            image_path,
            thumb_path,
            thumb_state,
            height: new_height,
            width: DEFAULT_IMG_WIDTH,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl ImageData {
    pub async fn new(image_path: PathBuf) -> image::error::ImageResult<Self> {
        let rgba_image = image_open(image_path).await?;
        Ok(Self {
            width: rgba_image.width(),
            height: rgba_image.height(),
            pixels: rgba_image.into_vec(),
        })
    }
}
