pub mod components;
pub mod fuu;
pub mod style;
pub mod types;

use crate::gui::components::{error_view, loading_page};
use crate::utils::*;
use fuu::Fuu;
use iced::font;
use iced::keyboard::{self, KeyCode};
use iced::widget::scrollable::Viewport;
use iced::{executor, window, Application, Command, Element, Event, Subscription, Theme};
use types::*;

#[derive(Debug, Clone)]
pub enum Message {
    WindowResize { width: u32, height: u32 },
    KeyPress(KeyCode),
    ChangeFocus(usize),
    Scrolled(Viewport),
    FontLoaded(Result<(), font::Error>),
    SourcesLoaded(Vec<ImageSource>),
    ThumbLoaded(Option<(u32,u32)>, usize),
    PreviewLoaded(Option<bytes::Bytes>, usize),
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
                font::load(include_bytes!("../../fonts/icons.otf").as_slice())
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
        match &self.current_page {
            Page::Loading => loading_page("loading ..."),
            Page::Gallery => self.gallery_view(),
            Page::ShowImage => self.image_preview(),
            Page::Error(err_msg) => error_view(err_msg),
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
