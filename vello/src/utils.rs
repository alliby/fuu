use std::io::Read;
use std::path::{Path, PathBuf};

// Check if a file is a valid image file
fn is_image(path: impl AsRef<Path>) -> bool {
    let mut buff = [0; 58];
    std::fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut buff))
        .map(|_| infer::is_image(&buff))
        .unwrap_or(false)
}

fn read_dir(path: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
    path.as_ref()
        .read_dir()
        .into_iter()
        .flatten()
        .filter_map(|entry_result| entry_result.ok())
        .map(|dir_entry| dir_entry.path())
}

pub fn read_args() -> Vec<PathBuf> {
    let mut sources = Vec::new();
    for path in std::env::args().skip(1).map(PathBuf::from) {
        if path.is_dir() {
            sources.extend(read_dir(path).filter(|img| is_image(img)))
        } else if is_image(&path) {
            sources.push(path)
        }
    }
    sources
}
