use bytes::Bytes;
use crate::gui::style::DEFAULT_IMG_WIDTH;
use crate::utils::*;
use std::path::PathBuf;
use std::hash::{Hash, Hasher};

#[derive(Default, Clone, Debug)]
pub enum ThumbState {
    #[default]
    Loading,
    Loaded,
    Error,
}

#[derive(Default, Clone, Debug)]
pub enum ImageState {
    #[default]
    Loading,
    Loaded(Bytes),
    Error,
}

#[derive(Clone, Debug)]
pub enum ImageSource {
    Path(PathBuf),
    Url(url::Url),
}

impl ImageSource {
    pub fn new<S: AsRef<str>>(input: S) -> Self {
        match url::Url::parse(input.as_ref()) {
            Ok(url) => Self::Url(url),
            Err(_) => Self::Path(PathBuf::from(input.as_ref())),
        }
    }

    pub fn as_path(&self) -> PathBuf {
        match self {
            Self::Url(url) => thumb_path(url.as_str()),
            Self::Path(pathbuf) => pathbuf.to_path_buf()
        }
    }
}

impl Default for ImageSource {
    fn default() -> Self {
        Self::Path(Default::default())
    }
}

#[derive(Debug, Clone)]
pub struct ImageCard {
    pub width: u32,
    pub height: u32,
    pub thumb: PathBuf,
    pub preview: ImageSource,
    pub thumb_state: ThumbState,
    pub preview_state: ImageState,
}


impl ImageCard {
    pub fn resize(&self, new_width: u32) -> (u32, u32) {
        let ratio = self.width as f32 / self.height as f32;
        let new_height = (new_width as f32 / ratio) as u32;
        (new_width, new_height)
    }

    fn from_path(image_path: PathBuf) -> Self {
        let thumb_path = thumb_path(&image_path);
        Self {
            preview: ImageSource::Path(image_path),
            thumb: thumb_path,
            ..Default::default()
        }
    }

    fn from_url(url: url::Url) -> Self {
        let thumb_path = thumb_path(thumb_path(url.as_str()));
        Self {
            preview: ImageSource::Url(url),
            thumb: thumb_path,
            ..Default::default()
        }
    }

    pub fn new(image_source: ImageSource) -> Self {
        match image_source {
            ImageSource::Path(path) => Self::from_path(path),
            ImageSource::Url(url) => Self::from_url(url),
        }
    }
}

impl Default for ImageCard {
    fn default() -> Self {
        Self {
            width: DEFAULT_IMG_WIDTH,
            height: DEFAULT_IMG_WIDTH,
            thumb: Default::default(),
            preview: Default::default(),
            thumb_state: Default::default(),
            preview_state: Default::default(),
        }
    }
}

impl std::cmp::PartialEq for ImageCard {
    fn eq(&self, other: &Self) -> bool {
        self.thumb == other.thumb
    }
}

impl std::cmp::Eq for ImageCard { }

impl Hash for ImageCard {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.thumb.hash(state);
    }
}
