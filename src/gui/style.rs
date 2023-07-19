use iced::widget::button;
use iced::Theme;

pub const CONTAINER_PADDING: u32 = 10;
pub const COLUMN_SPACING: u32 = 10;
pub const ROW_SPACING: u32 = 5;
pub const DEFAULT_IMG_WIDTH: u32 = 200;

#[derive(Default)]
pub enum ImageCard {
    #[default]
    Normal,
    Selected,
}

impl button::StyleSheet for ImageCard {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let palette = style.extended_palette();

        match self {
            ImageCard::Normal => button::Appearance {
                border_color: palette.secondary.base.color,
                border_width: 2.0,
                ..Default::default()
            },
            ImageCard::Selected => button::Appearance {
                border_color: palette.primary.strong.color,
                border_width: 2.0,
                ..Default::default()
            },
        }
    }
}
