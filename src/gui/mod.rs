pub mod components;
pub mod fuu;
pub mod style;
pub mod types;
pub mod widgets;

use crate::gui::components::{error_view, welcome_page};
use crate::gui::widgets::modal::Modal;
use crate::utils::*;
use fuu::Fuu;
use iced::font;
use iced::keyboard::{self, KeyCode};
use iced::{executor, window, Application, Command, Element, Event, Subscription, Theme};
use iced::widget::{text, container};
use std::path::PathBuf;
use types::*;

#[derive(Debug, Clone)]
pub enum Message {
    WindowResize { width: u32, height: u32 },
    KeyPress(KeyCode),
    ChangeFocus(usize),
    FontLoaded(Result<(), font::Error>),
    SourcesLoaded(Vec<ImageSource>),
    ThumbLoaded(Option<(u32,u32)>, usize),
    PreviewLoaded(Option<bytes::Bytes>, usize),
    FileDropped(PathBuf),
    FileHovered,
    HideOverlay,
    LoadThumbs,
}

impl Application for Fuu {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = Vec<ImageSource>;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self::new(),
            Command::batch([
                font::load(include_bytes!("../../fonts/icons-subset.ttf").as_slice())
                    .map(Message::FontLoaded),
                font::load(include_bytes!("../../fonts/japanese-subset.ttf").as_slice())
                    .map(Message::FontLoaded),
                Command::perform(
                    async {
                        create_cache_dir().await.expect("Cannot create cache dir");
                        read_sources(flags).await
                    },
                    Message::SourcesLoaded,
                ),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("Fuu")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        self.update(message)
    }

    fn view(&self) -> Element<Message> {
        let content = match &self.current_page {
            Page::Welcome => welcome_page(),
            Page::Gallery => self.gallery_view(),
            Page::ShowImage => self.image_preview(),
            Page::Error(err_msg) => error_view(err_msg),
        };
        if self.file_drag {
            let overlay = container(text("File Hovered"))
                .width(self.container_dim.0 as u16 / 2)
                .height(self.container_dim.1 as u16 / 2)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Custom(Box::new(style::ModalStyle)));
            Modal::new(content, overlay)
                .on_blur(Message::HideOverlay)
                .into()
        } else {
            content
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced::subscription::events_with(|event, _| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
                Some(Message::KeyPress(key_code))
            }
            Event::Window(window::Event::Resized { width, height }) => {
                Some(Message::WindowResize { width, height })
            }
            Event::Window(window::Event::FileDropped(file_path)) => {
                Some(Message::FileDropped(file_path))
            }
            Event::Window(window::Event::FileHovered(_)) => {
                Some(Message::FileHovered)
            }
            Event::Window(window::Event::FilesHoveredLeft) => {
                Some(Message::HideOverlay)
            }
            _ => None,
        })
    }
}
