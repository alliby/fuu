#[derive(Default)]
pub enum Page {
    #[default]
    Loading,
    Gallery,
    ShowImage,
    Error(String),
}
