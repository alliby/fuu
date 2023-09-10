#[derive(Default)]
pub enum Page {
    #[default]
    Welcome,
    Gallery,
    Selection,
    ShowImage,
    Error(String),
}
