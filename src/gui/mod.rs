pub mod fuu;
pub mod style;
pub mod types;
pub mod components;

use crate::utils::{create_cache_dir, read_dir};
use crate::gui::components::{ error_view, loading_page };
use fuu::Fuu;
use iced::font;
use iced::keyboard::{self, KeyCode};
use iced::widget::scrollable::Viewport;
use iced::widget::{container, text};
use iced::{executor, window, Application, Command, Element, Event, Length, Subscription, Theme};
use std::path::PathBuf;
use types::{ImageData, Page};

#[derive(Debug, Clone)]
pub enum Message {
    WindowResize { width: u32, height: u32 },
    KeyPress(KeyCode),
    ChangeFocus(usize),
    Scrolled(Viewport),
    FontLoaded(Result<(), font::Error>),
    PathsLoaded(Result<Vec<PathBuf>, String>),
    ThumbLoaded(usize),
    ImageLoaded(Option<ImageData>, usize),
    LoadThumbs,
}

impl Application for Fuu {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = PathBuf;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self::new(),
            Command::batch([
                font::load(include_bytes!("../../fonts/icons.otf").as_slice())
                    .map(Message::FontLoaded),
                Command::perform(
                    async {
                        create_cache_dir().await?;
                        read_dir(flags).await
                    },
                    |result| Message::PathsLoaded(result.map_err(|err| err.to_string())),
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
        match &self.current_page {
            Page::Loading => loading_page("loading ..."),
            Page::Gallery => self.gallery_view(),
            Page::ShowImage => self.image_preview(),
            Page::Error(err_msg) => error_view(err_msg)
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
            _ => None,
        })
    }
}
