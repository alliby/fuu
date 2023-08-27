use iced::alignment;
use iced::font::Font;
use iced::widget::{text, Text};

const ICONS: Font = Font::with_name("Toss Face Font Mac");

fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
}

pub fn arrow_left_icon() -> Text<'static> {
    icon('\u{2B05}')
}

pub fn arrow_right_icon() -> Text<'static> {
    icon('\u{27A1}')
}

pub fn flower() -> Text<'static> {
    icon('\u{1F338}')
}
