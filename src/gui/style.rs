use iced::widget::{button, container};
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

#[derive(Default)]
pub struct ModalStyle;

impl container::StyleSheet for ModalStyle {
    type Style = Theme;
    
    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let palette = style.palette();

        container::Appearance {
            text_color: palette.text.into(),
            background: Some(palette.primary.into()),
            border_radius: 10.0.into(),
            ..Default::default()
        }
    }
}
