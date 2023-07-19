use crate::gui::Message;
use iced::{Element, Length};
use iced::widget::{text, container};

pub fn loading_page(message: &str) -> Element<Message> {
    container(text(message))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

