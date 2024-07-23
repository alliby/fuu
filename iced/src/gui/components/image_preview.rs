use crate::gui::components::icons::{arrow_left_icon, arrow_right_icon};
use crate::gui::types::{ImageCard, ImageState};
use crate::gui::Message;

use iced::keyboard::KeyCode;
use iced::widget::image::{viewer, Handle};
use iced::widget::{button, container, row, text};
use iced::{alignment, theme, Element, Length};

pub fn image_preview(image_card: &ImageCard, dim: (u32, u32)) -> Element<Message> {
    let image = match &image_card.preview_state {
        ImageState::Loaded(bytes) => Element::new(
            viewer(Handle::from_memory(bytes.clone()))
                .width(Length::Fill)
                .height(Length::Fill)
        ),
        ImageState::Loading => Element::new(
            container(text("loading ..."))
                .height(dim.1 as u16)
                .width(Length::Fill)
                .center_x()
                .center_y()
        ),
        ImageState::Error => Element::new(
            container(text("error"))
                .height(dim.1 as u16)
                .width(Length::Fill)
                .center_x()
                .center_y()
        ),
    };
    let content = row![
        button(arrow_left_icon())
            .on_press(Message::KeyPress(KeyCode::Left))
            .height(dim.1 as u16)
            .style(theme::Button::Text),
        image,
        button(arrow_right_icon())
            .on_press(Message::KeyPress(KeyCode::Right))
            .height(dim.1 as u16)
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
