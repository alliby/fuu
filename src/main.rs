pub mod gui;
pub mod utils;

use gui::fuu::Fuu;
use gui::types::ImageSource;
use iced::{Application, Settings};
use std::env;

fn main() -> iced::Result {
    let sources: Vec<ImageSource> = env::args().skip(1).map(ImageSource::new).collect();
    Fuu::run(Settings::with_flags(sources))
}
