#[derive(Default)]
pub enum Page {
    #[default]
    Welcome,
    Gallery,
    ShowImage,
    Error(String),
}
