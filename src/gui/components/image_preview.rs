use crate::gui::components::icons::{arrow_left_icon, arrow_right_icon};
use crate::gui::types::{ImageCard, ImageState};
use crate::gui::Message;

use iced::keyboard::KeyCode;
use iced::widget::image::{viewer, Handle};
use iced::widget::{button, container, row, text, Space};
use iced::{alignment, theme, Element, Length};

pub fn image_preview(image_card: &ImageCard, dim: (u32, u32)) -> Element<Message> {
    let (width, height) = dim;
    let content = row![
        button(arrow_left_icon())
            .on_press(Message::KeyPress(KeyCode::Left))
            .height(height as u16)
            .style(theme::Button::Text),
        Space::with_width(Length::Fill),
        match &image_card.preview_state {
            ImageState::Loaded(bytes) => Element::new(
                viewer(Handle::from_memory(bytes.clone())).width(width as u16 - 60)
            ),
            ImageState::Loading => Element::new(text("loading ...")),
            ImageState::Error => Element::new(text("error")),
        },
        Space::with_width(Length::Fill),
        button(arrow_right_icon())
            .on_press(Message::KeyPress(KeyCode::Right))
            .height(height as u16)
            .style(theme::Button::Text),
    ]
    .align_items(alignment::Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
}
