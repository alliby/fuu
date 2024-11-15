use crate::gui::components::image_preview;
use crate::gui::style;
use crate::gui::types::*;
use crate::gui::Message;
use crate::utils::*;

use iced::keyboard::KeyCode;
use iced::widget::image::{Handle, Image};
use iced::widget::scrollable::AbsoluteOffset;
use iced::widget::{button, column, container, row, scrollable, text, Button};
use iced::{theme, Command, Element};
use indexmap::IndexSet;
use once_cell::sync::Lazy;
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use style::{COLUMN_SPACING, CONTAINER_PADDING, DEFAULT_IMG_WIDTH, ROW_SPACING};

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static COMMAND_COUNTER: AtomicUsize = AtomicUsize::new(0);
const COMMANDS_NUM: usize = 4;

#[derive(Default)]
pub struct Fuu {
    pub file_drag: bool,
    pub show_selections: bool,
    pub current_page: Page,
    pub images: Vec<ImageCard>,
    pub container_dim: (u32, u32),
    pub img_width: u32,
    pub selected: usize,
    pub selections_list: IndexSet<usize>,
    pub current_scroll_offset: scrollable::AbsoluteOffset,
}

impl Fuu {
    pub fn new() -> Self {
        Self {
            img_width: DEFAULT_IMG_WIDTH,
            ..Default::default()
        }
    }

    pub fn row_num(&self) -> usize {
        let container_width = self.container_dim.0.max(self.img_width + CONTAINER_PADDING);
        ((container_width - CONTAINER_PADDING) / self.img_width) as usize
    }

    pub fn get_top(&self) -> usize {
        let row_num = self.row_num();
        if self.selected >= row_num {
            self.selected - row_num
        } else {
            self.selected
        }
    }

    pub fn get_bottom(&self) -> usize {
        let row_num = self.row_num();
        if self.selected + row_num < self.images.len() {
            self.selected + row_num
        } else {
            self.selected
        }
    }

    pub fn get_forward(&self) -> usize {
        (self.selected + 1).min(self.images.len() - 1)
    }

    pub fn get_backward(&self) -> usize {
        self.selected.max(1) - 1
    }

    pub fn height_from_top(&self) -> u32 {
        let row_num = self.row_num();
        let current_row = self.selected % row_num;
        CONTAINER_PADDING
            + self.images[current_row..self.selected]
                .iter()
                .step_by(row_num)
                .fold(0, |height, image| {
                    height + image.resize(self.img_width).1 + COLUMN_SPACING
                })
    }

    pub fn height_from_bottom(&self) -> u32 {
        let row_num = self.row_num();
        let current_row = self.selected % row_num;
        CONTAINER_PADDING
            + self.images[current_row..(self.selected + 1).min(self.images.len())]
                .iter()
                .step_by(row_num)
                .fold(0, |height, image| {
                    height + image.resize(self.img_width).1 + 2 * COLUMN_SPACING
                })
    }

    pub fn calculate_scroll_offset(&self) -> AbsoluteOffset {
        let height_from_bottom = self.height_from_bottom();
        let height_from_top = self.height_from_top();
        let height = self.container_dim.1;
        if height_from_bottom > height + self.current_scroll_offset.y as u32 {
            AbsoluteOffset {
                y: (height_from_bottom + self.images[self.selected].height - height) as f32,
                ..Default::default()
            }
        } else if height_from_top < self.current_scroll_offset.y as u32 {
            AbsoluteOffset {
                y: (height_from_top - CONTAINER_PADDING) as f32,
                ..Default::default()
            }
        } else {
            self.current_scroll_offset
        }
    }

    fn update_scroll_offset(&mut self) -> Command<Message> {
        self.current_scroll_offset = self.calculate_scroll_offset();
        if let Page::Gallery = self.current_page {
            return scrollable::scroll_to(SCROLLABLE_ID.clone(), self.current_scroll_offset);
        }
        Command::none()
    }

    fn update_preview_data(&self) -> Command<Message> {
        let index = if self.show_selections {
            self.selections_list[self.selected]
        } else {
            self.selected
        };
        let image_card = &self.images[index];
        match image_card.preview_state {
            ImageState::Loading => {
                let source = image_card.preview.clone();
                Command::perform(fetch_source(source), move |rgba_image| {
                    Message::PreviewLoaded(rgba_image, index)
                })
            }
            _ => Command::none(),
        }
    }

    pub fn image_preview(&self) -> Element<Message> {
        let image_card = if self.show_selections {
            let index = self.selected.min(self.selections_list.len() - 1);
            &self.images[self.selections_list[index]]
        } else {
            &self.images[self.selected]
        };
        image_preview(image_card, self.container_dim)
    }

    fn card_style(&self, index: usize) -> theme::Button {
        if index == self.selected {
            return theme::Button::Custom(Box::new(style::ImageCard::Hovered));
        }
        if self.show_selections || self.selections_list.contains(&index) {
            return theme::Button::Custom(Box::new(style::ImageCard::Selected));
        }
        theme::Button::Custom(Box::new(style::ImageCard::Normal))
    }

    fn card_view(&self, index: usize) -> Button<Message> {
        let image_card = if self.show_selections {
            &self.images[self.selections_list[index]]
        } else {
            &self.images[index]
        };
        let (w, h) = image_card.resize(self.img_width);
        let content = match &image_card.thumb_state {
            ThumbState::Loading => Element::new(
                container(text("Loading ...").style(iced::Color::WHITE))
                    .width(w as u16)
                    .height(h as u16)
                    .center_x()
                    .center_y(),
            ),
            ThumbState::Error => Element::new(
                container(text("Error loading image").style(iced::Color::WHITE))
                    .width(w as u16)
                    .height(h as u16)
                    .center_x()
                    .center_y(),
            ),
            ThumbState::Loaded => Element::new(
                Image::new(Handle::from_path(&image_card.thumb))
                    .width(w as u16)
                    .height(h as u16),
            ),
        };
        button(content)
            .on_press(Message::ChangeFocus(index))
            .style(self.card_style(index))
    }

    pub fn gallery_view(&self) -> Element<Message> {
        let row_num = self.row_num();
        let elem_num = if self.show_selections {
            self.selections_list.len()
        } else {
            self.images.len()
        };
        let mut remaining = elem_num % row_num;
        let mut rows = row![]
            .spacing(ROW_SPACING as u16)
            .padding(CONTAINER_PADDING as u16);

        for i in 0..row_num {
            let mut columns = column![].spacing(COLUMN_SPACING as u16);
            let mut column_num = elem_num / row_num;
            if remaining != 0 {
                remaining -= 1;
                column_num += 1;
            }
            for j in 0..column_num {
                columns = columns.push(self.card_view(i + j * row_num));
            }
            rows = rows.push(columns);
        }

        let container_width = self.container_dim.0.max(self.img_width + CONTAINER_PADDING) as u16;

        let content = container(rows)
            .width(container_width - CONTAINER_PADDING as u16)
            .center_x();

        scrollable(content)
            .id(SCROLLABLE_ID.clone())
            .width(container_width)
            .height(self.container_dim.1 as u16)
            .into()
    }

    fn handle_keypress(&mut self, key: KeyCode) -> Command<Message> {
        if let Page::Welcome | Page::Error(_) = self.current_page {
            return Command::none()
        }
        match key {
            KeyCode::Plus | KeyCode::NumpadAdd => {
                self.img_width += 20;
                self.img_width = self.img_width.min(self.container_dim.0);
                return self.update_scroll_offset();
            }
            KeyCode::Minus | KeyCode::NumpadSubtract => {
                self.img_width -= 20;
                self.img_width = self.img_width.max(DEFAULT_IMG_WIDTH / 2);
                return self.update_scroll_offset();
            }
            KeyCode::Left | KeyCode::P => {
                self.selected = self.get_backward();
                if self.show_selections {
                    self.selected = self.selected.min(self.selections_list.len() - 1)
                }
                match self.current_page {
                    Page::Gallery => return self.update_scroll_offset(),
                    Page::ShowImage => return self.update_preview_data(),
                    _ => (),
                }
            }
            KeyCode::Right | KeyCode::N => {
                self.selected = self.get_forward();
                if self.show_selections {
                    self.selected = self.selected.min(self.selections_list.len() - 1)
                }
                match self.current_page {
                    Page::Gallery => return self.update_scroll_offset(),
                    Page::ShowImage => return self.update_preview_data(),
                    _ => (),
                }
            }
            KeyCode::Up => {
                self.selected = self.get_top();
                if self.show_selections {
                    self.selected = self.selected.min(self.selections_list.len() - 1)
                }
                if let Page::Gallery = self.current_page {
                    return self.update_scroll_offset();
                }
            }
            KeyCode::Down => {
                self.selected = self.get_bottom();
                if self.show_selections {
                    self.selected = self.selected.min(self.selections_list.len() - 1)
                }
                if let Page::Gallery = self.current_page {
                    return self.update_scroll_offset();
                }
            }
            KeyCode::Equals | KeyCode::Key0 => {
                self.img_width = self.container_dim.0 / 5;
                return self.update_scroll_offset();
            }
            KeyCode::Enter => match self.current_page {
                Page::Gallery => {
                    self.current_page = Page::ShowImage;
                    return self.update_preview_data();
                }
                Page::ShowImage => {
                    self.current_page = Page::Gallery;
                    return self.update_scroll_offset();
                }
                _ => (),
            },
            KeyCode::M => {
                let index = if self.show_selections {
                    self.selections_list[self.selected]
                } else {
                    self.selected
                };
                if !self.selections_list.insert(index) {
                    self.selections_list.remove(&index);
                }
            }
            KeyCode::Space => {
                if let Page::Gallery = self.current_page {
                    self.show_selections ^= true;
                    self.selected = 0;
                }
            }
            KeyCode::Escape => {
                match self.current_page {
                    Page::Gallery => if self.show_selections {
                        self.show_selections = false;
                        self.selected = self.selections_list[self.selected];
                    }
                    Page::ShowImage => {
                        self.current_page = Page::Gallery;
                    }
                    _ => return Command::perform(async {}, |_| Message::CloseRequested)
                }
            }
            _ => (),
        }
        Command::none()
    }
    
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CloseRequested => {
                let mut stdout = io::stdout().lock();
                for index in &self.selections_list {
                    let source_path = self.images[*index].preview.as_path();
                    if source_path.exists() {
                        writeln!(&mut stdout, "{}", source_path.display()).unwrap()
                    }
                }
                std::process::exit(0)
            }
            Message::KeyPress(key) => return self.handle_keypress(key),
            Message::WindowResize { width, height } => {
                self.container_dim = (width, height);
                self.img_width = width / 5;
                return self.update_scroll_offset();
            }
            Message::ChangeFocus(selected) => {
                if self.selected == selected {
                    self.current_page = Page::ShowImage;
                    return self.update_preview_data();
                }
                self.selected = selected;
                return self.update_scroll_offset();
            }
            Message::SourcesLoaded(sources) => {
                let mut image_cards = IndexSet::with_capacity(self.images.len());
                image_cards.extend(
                    self.images
                        .drain(..)
                        .chain(sources.into_iter().map(ImageCard::new)),
                );
                self.images = image_cards.into_iter().collect();
                if !self.images.is_empty() {
                    self.current_page = Page::Gallery;
                    return Command::perform(async {}, |_| Message::LoadThumbs);
                }
                self.current_page = Page::Welcome;
            }
            Message::LoadThumbs => {
                return Command::batch(
                    self.images
                        .iter()
                        .enumerate()
                        .filter(|(_, image_card)| {
                            matches!(image_card.thumb_state, ThumbState::Loading)
                        })
                        .take(COMMANDS_NUM)
                        .map(|(i, image_card)| {
                            Command::perform(generate_thumb(image_card.clone()), move |dim| {
                                Message::ThumbLoaded(dim, i)
                            })
                        }),
                )
            }
            Message::ThumbLoaded(Some(dim), index) => {
                let image_card = &mut self.images[index];
                (image_card.width, image_card.height) = dim;
                image_card.thumb_state = ThumbState::Loaded;
                let counter = COMMAND_COUNTER.load(Ordering::Relaxed) + 1;
                COMMAND_COUNTER.store(counter, Ordering::Relaxed);
                if counter == COMMANDS_NUM {
                    COMMAND_COUNTER.store(0, Ordering::Relaxed);
                    return Command::perform(async {}, |_| Message::LoadThumbs);
                }
            }
            Message::ThumbLoaded(None, index) => {
                self.images[index].thumb_state = ThumbState::Error;
            }
            Message::PreviewLoaded(Some(rgba_image), index) => {
                if self.selected == index || self.selections_list.contains(&index) {
                    self.images[index].preview_state = ImageState::Loaded(rgba_image);
                }
            }
            Message::PreviewLoaded(None, index) => {
                self.images[index].preview_state = ImageState::Error;
            }
            Message::FileDropped(file_path) => {
                self.file_drag = false;
                let sources = ImageSource::Path(file_path);
                return Command::perform(read_sources(vec![sources]), Message::SourcesLoaded);
            }
            Message::FileHovered => {
                self.file_drag = true;
            }
            Message::HideOverlay => {
                self.file_drag = false;
            }
            _ => (),
        }
        Command::none()
    }
}
