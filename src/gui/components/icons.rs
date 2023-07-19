use iced::font::Font;
use iced::widget::{ text, Text };
use iced::alignment;

const ICONS: Font = Font::with_name("Font Awesome 6 Free Regular");

fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
}

pub fn arrow_left_icon() -> Text<'static> {
    icon('\u{F060}')
}

pub fn arrow_right_icon() -> Text<'static> {
    icon('\u{F061}')
}

