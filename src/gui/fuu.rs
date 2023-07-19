use crate::gui::style;
use crate::gui::types::{ImageCard, ImageData, ImageState, Page, ThumbState};
use crate::gui::Message;
use crate::utils::generate_thumb;
use crate::gui::components::image_preview;

use iced::keyboard::KeyCode;
use iced::widget::image::{Handle, Image};
use iced::widget::scrollable::{AbsoluteOffset, Properties};
use iced::widget::{button, column, container, row, scrollable, Button};
use iced::{theme, Command, Element};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicUsize, Ordering};
use style::{COLUMN_SPACING, CONTAINER_PADDING, DEFAULT_IMG_WIDTH, ROW_SPACING};

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static MEMORY_USAGE: AtomicUsize = AtomicUsize::new(0);
static COMMAND_COUNTER: AtomicUsize = AtomicUsize::new(0);

const COMMANDS_NUM: usize = 4;
const MAX_MEM_USE: usize = 209715200; // 200MB

#[derive(Default)]
pub struct Fuu {
    pub current_page: Page,
    pub images: Vec<ImageCard>,
    pub container_dim: (u32, u32),
    pub img_width: u32,
    pub selected: usize,
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

    fn update_image_data(&self) -> Command<Message> {
        let index = self.selected;
        let image_card = &self.images[index];
        if let ImageState::Loading = image_card.image_state {
            let image_path = image_card.image_path.clone();
            return Command::perform(ImageData::new(image_path), move |data| {
                Message::ImageLoaded(data.ok(), index)
            });
        }
        Command::none()
    }

    pub fn image_preview(&self) -> Element<Message> {
        image_preview(&self.images[self.selected], self.container_dim)
    }
    
    fn card_view(&self, index: usize) -> Button<Message> {
        let image_card = &self.images[index];
        let (w, h) = image_card.resize(self.img_width);
        button(match &image_card.thumb_state {
            ThumbState::Loading => Element::new(column![].width(w as u16).height(h as u16)),
            ThumbState::Loaded => Element::new(
                Image::new(Handle::from_path(&image_card.thumb_path))
                    .width(w as u16)
                    .height(h as u16),
            )
        })
        .on_press(Message::ChangeFocus(index))
        .style(if index == self.selected {
            theme::Button::Custom(Box::new(style::ImageCard::Selected))
        } else {
            theme::Button::Custom(Box::new(style::ImageCard::Normal))
        })
    }

    pub fn gallery_view(&self) -> Element<Message> {
        let row_num = self.row_num();
        let elem_num = self.images.len();
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
            .vertical_scroll(Properties::new().scroller_width(5).width(4.5))
            .id(SCROLLABLE_ID.clone())
            .on_scroll(Message::Scrolled)
            .width(container_width)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::KeyPress(key) => match key {
                KeyCode::Plus | KeyCode::NumpadAdd => {
                    self.img_width += 20;
                    self.img_width = self.img_width.min(self.container_dim.0);
                    self.update_scroll_offset()
                }
                KeyCode::Minus | KeyCode::NumpadSubtract => {
                    self.img_width -= 20;
                    self.img_width = self.img_width.max(100);
                    self.update_scroll_offset()
                }
                KeyCode::Left | KeyCode::P => {
                    self.selected = self.get_backward();
                    if let Page::ShowImage = self.current_page {
                        return self.update_image_data();
                    }
                    self.update_scroll_offset()
                }
                KeyCode::Right | KeyCode::N => {
                    self.selected = self.get_forward();
                    if let Page::ShowImage = self.current_page {
                        return self.update_image_data();
                    }
                    self.update_scroll_offset()
                }
                KeyCode::Up => {
                    if let Page::Gallery = self.current_page {
                        self.selected = self.get_top();
                    }
                    self.update_scroll_offset()
                }
                KeyCode::Down => {
                    if let Page::Gallery = self.current_page {
                        self.selected = self.get_bottom();
                    }
                    self.update_scroll_offset()
                }
                KeyCode::Equals | KeyCode::Key0 => {
                    self.img_width = self.container_dim.0 / 5;
                    self.update_scroll_offset()
                }
                KeyCode::Enter => match self.current_page {
                    Page::Gallery => {
                        self.current_page = Page::ShowImage;
                        self.update_image_data()
                    }
                    Page::ShowImage => {
                        self.current_page = Page::Gallery;
                        self.update_scroll_offset()
                    }
                    _ => Command::none(),
                },
                _ => Command::none(),
            },
            Message::WindowResize { width, height } => {
                self.container_dim = (width, height);
                self.img_width = width / 5;
                self.update_scroll_offset()
            }
            Message::ChangeFocus(selected) => {
                if self.selected == selected {
                    self.current_page = Page::ShowImage;
                    return self.update_image_data();
                }
                self.selected = selected;
                self.update_scroll_offset()
            }
            Message::Scrolled(viewport) => {
                self.current_scroll_offset = viewport.absolute_offset();
                Command::none()
            }
            Message::PathsLoaded(Ok(paths)) => {
                self.images = paths.into_iter().map(ImageCard::new).collect();
                self.current_page = Page::Gallery;
                Command::perform(async {}, move |_| Message::LoadThumbs)
            }
            Message::PathsLoaded(Err(io_error)) => {
                self.current_page = Page::Error(io_error);
                Command::none()
            }
            Message::LoadThumbs => Command::batch(
                self.images
                    .iter()
                    .enumerate()
                    .filter(|(_,image_card)| matches!(image_card.thumb_state, ThumbState::Loading))
                    .take(COMMANDS_NUM)
                    .map(|(i, image_card)| {
                        let image_path = image_card.image_path.clone();
                        let thumb_path = image_card.thumb_path.clone();
                        let resize_dim = (image_card.width, image_card.height);
                        Command::perform(
                            generate_thumb(image_path, thumb_path, resize_dim),
                            move |_| Message::ThumbLoaded(i),
                        )
                    }),
            ),
            Message::ThumbLoaded(index) => {
                self.images[index].thumb_state = ThumbState::Loaded;
                let counter = COMMAND_COUNTER.load(Ordering::Relaxed) + 1;
                COMMAND_COUNTER.store(counter, Ordering::Relaxed);
                if counter == COMMANDS_NUM {
                    COMMAND_COUNTER.store(0, Ordering::Relaxed);
                    return Command::perform(async {}, |_| Message::LoadThumbs);
                }
                Command::none()
            }
            Message::ImageLoaded(None, index) => {
                self.images[index].image_state = ImageState::Error;
                Command::none()
            }
            Message::ImageLoaded(Some(img_data), index) => {
                if self.selected != index {
                    return Command::none();
                }
                let memory_use = img_data.width * img_data.height * 4;
                self.images[index].image_state = ImageState::Loaded(img_data);
                let counter = MEMORY_USAGE.load(Ordering::Relaxed) + memory_use as usize;
                MEMORY_USAGE.store(counter, Ordering::Relaxed);
                if counter > MAX_MEM_USE {
                    self.images
                        .iter_mut()
                        .filter(|image| matches!(image.image_state, ImageState::Loaded(_)))
                        .for_each(|image| image.image_state = ImageState::Loading);
                    MEMORY_USAGE.store(0, Ordering::Relaxed);
                }
                Command::none()
            }
            _ => Command::none(),
        }
    }
}
