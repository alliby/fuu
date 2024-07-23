use crate::gui::Message;
use iced::widget::{container, text};
use iced::{Element, Length};

pub fn error_view(error_message: &str) -> Element<Message> {
    container(text(error_message))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}
