use crate::gui::Message;
use iced::widget::{container, text};
use iced::{Element, Length};

pub fn loading_page(message: &str) -> Element<Message> {
    container(text(message))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}
