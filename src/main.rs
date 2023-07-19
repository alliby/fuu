pub mod gui;
pub mod utils;

use gui::fuu::Fuu;
use iced::{Application, Settings};
use std::env;
use std::path::Path;

fn main() -> iced::Result {
    let args: String = env::args().nth(1).unwrap();

    Fuu::run(Settings::with_flags(Path::new(&args).into()))
}
