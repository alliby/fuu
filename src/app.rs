use super::Theme;
use std::path::PathBuf;
use std::time::Instant;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::keyboard::Key;

const WINDOW_WIDTH: u32 = 1044;
const WINDOW_HEIGHT: u32 = 800;

const DEFAULT_COL_NUM: usize = 2;
const DEFAULT_ROW_NUM: usize = 1;
const MIN_AXE_NUM: usize = 1;
const MAX_AXE_NUM: usize = 10;

#[derive(Copy, Clone, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub enum Page {
    Gallery,
    Preview,
}

impl Page {
    pub fn next(&self) -> Self {
        match self {
            Self::Gallery => Self::Preview,
            Self::Preview => Self::Gallery,
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    pub active: usize,
    pub active_theme: usize,
    pub page: Page,
    pub col_num: usize,
    pub row_num: usize,
    pub images: Vec<PathBuf>,
    pub window_size: (u32, u32),
    pub start_time: Instant,
}

impl AppState {
    pub fn new(images: Vec<PathBuf>) -> Self {
        Self {
            images,
            active: 0,
            active_theme: 0,
            page: Page::Gallery,
            col_num: DEFAULT_COL_NUM,
            row_num: DEFAULT_ROW_NUM,
            window_size: (WINDOW_WIDTH, WINDOW_HEIGHT),
            start_time: Instant::now(),
        }
    }

    fn navigate(&mut self, direction: Direction, num: usize) {
        let new_active = match direction {
            Direction::Up => self.active.saturating_sub(self.col_num * num),
            Direction::Right => (self.active + num).min(self.images.len() - 1),
            Direction::Down => (self.active + num * self.col_num).min(self.images.len() - 1),
            Direction::Left => self.active.saturating_sub(num),
        };

        self.active = if new_active == self.active {
            match direction {
                Direction::Up | Direction::Left => self.images.len() - 1,
                Direction::Right | Direction::Down => 0,
            }
        } else {
            new_active
        };
    }
}

pub fn handle_key(app: &mut AppState, event: &KeyEvent) -> bool {
    use winit::keyboard::NamedKey::*;
    use Direction::*;

    if event.state != ElementState::Pressed {
        return false;
    }

    match (app.page, event.logical_key.as_ref()) {
        (_, Key::Character(char)) if char == "q" => std::process::exit(0),
        (_, Key::Character(char)) if char == "s" => {
            app.active_theme = (app.active_theme + 1) % Theme::ALL.len();
        }
        (_, Key::Character(char)) if char == "S" => {
            if app.active_theme == 0 {
                app.active_theme = Theme::ALL.len() - 1;
                return true;
            }
            app.active_theme = app.active_theme.saturating_sub(1) % Theme::ALL.len()
        }
        (Page::Gallery, Key::Character(char)) if char == "+" => {
            app.col_num = (app.col_num - 1).max(MIN_AXE_NUM);
            app.row_num = (app.row_num - 1).max(MIN_AXE_NUM);
        }
        (Page::Gallery, Key::Character(char)) if char == "-" => {
            app.col_num = (app.col_num + 1).min(MAX_AXE_NUM);
            app.row_num = (app.row_num + 1).min(MAX_AXE_NUM);
        }

        (_, Key::Named(ArrowRight)) => app.navigate(Right, 1),
        (_, Key::Named(ArrowLeft)) => app.navigate(Left, 1),
        (_, Key::Named(ArrowUp)) => app.navigate(Up, 1),
        (_, Key::Named(ArrowDown)) => app.navigate(Down, 1),
        (Page::Gallery, Key::Named(PageUp)) => app.navigate(Up, app.col_num),
        (Page::Gallery, Key::Named(PageDown)) => app.navigate(Down, app.col_num),
        (_, Key::Named(Enter)) => app.page = app.page.next(),
        _ => return false,
    }
    true
}
