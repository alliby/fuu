use crate::gui::Message;
use crate::gui::components::icons::flower;
use iced::widget::{container, text, row};
use iced::{Element, Length};
use iced::Font;

const TEXT: Font = Font::with_name("Noto Sans JP");

pub fn welcome_page() -> Element<'static, Message> {
    container(row![
        flower().size(40.),
        text(" Fuu - ").size(50.),
        text("フウ").font(TEXT).size(50.)
    ].align_items(iced::Alignment::End))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}
