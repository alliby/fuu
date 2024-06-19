use super::{AppState, Page, Theme};
use crate::image::*;

use vello::kurbo::{Affine, CircleSegment, Rect, RoundedRect, Stroke};
use vello::peniko::{Color, Fill, Image};
use vello::Scene;

use std::f64::consts::PI;

const BAR_WIDTH: f64 = 5.0;
const BAR_PAD_X: f64 = 10.0;
const BAR_PAD_Y: f64 = 5.0;
const BAR_GAP: f64 = 15.0;

const GRID_PAD_X: f64 = 13.0;
const GRID_PAD_Y: f64 = 15.0;
const GRID_GAP_X: f64 = 15.0;
const GRID_GAP_Y: f64 = 13.0;

const CARD_THICK: f64 = 5.0;

pub fn draw(scene: &mut Scene, app: &AppState) -> bool {
    scene.reset();
    let mut redraw = false;
    match app.page {
        Page::Gallery => {
            draw_scrollbar(scene, &app);
            redraw |= draw_grid(scene, &app);
        }
        Page::Preview => {
            redraw |= draw_preview(scene, &app);
        }
    }
    redraw
}

fn draw_preview(scene: &mut Scene, app: &AppState) -> bool {
    let mut redraw = false;
    let (width, height) = app.window_size;
    let rect = Rect::new(0.0, 0.0, width as f64, height as f64);
    let image_state = get_image(&app.images[app.active], ImageSize::Preview);
    match image_state {
        ImageState::Loading => {
            let color = Theme::ALL[app.active_theme].highlight;
            let time = app.start_time.elapsed().as_secs_f64();
            draw_spinner(scene, rect, Affine::IDENTITY, time, color);
            redraw = true;
        }
        ImageState::Loaded(ref image) => {
            draw_image_viewer(scene, image, rect);
        }
    }
    redraw
}

fn draw_image_viewer(scene: &mut Scene, image: &Image, rect: Rect) {
    let size = rect.size();
    let cw = size.width;
    let ch = size.height;
    let scale = (cw / image.width as f64).min(ch / image.height as f64);
    let sw = image.width as f64 * scale;
    let sh = image.height as f64 * scale;
    scene.fill(
        Fill::NonZero,
        Affine::translate(((cw - sw) / 2.0, (ch - sh) / 2.0)),
        image,
        Affine::scale(scale).into(),
        &rect,
    );
}

fn draw_scrollbar(scene: &mut Scene, app: &AppState) {
    let (width, height) = app.window_size;
    let item_count = app.col_num * app.row_num;
    let current_page = app.active / item_count;
    let bar_count = (app.images.len() as f64 / item_count as f64).ceil();
    if bar_count == 1.0 {
        return;
    }
    let bar_gap = BAR_GAP / bar_count;
    let bar_height = ((height as f64 - 2.0 * BAR_PAD_Y) / bar_count) - bar_gap;
    let rect = RoundedRect::new(
        width as f64 - BAR_PAD_X - BAR_WIDTH,
        BAR_PAD_Y,
        width as f64 - BAR_PAD_X,
        BAR_PAD_Y + bar_height,
        3.0,
    );
    for i in 0..bar_count as usize {
        let translate_y = (bar_height + bar_gap) * i as f64;
        let color = if i == current_page {
            Theme::ALL[app.active_theme].highlight
        } else {
            Theme::ALL[app.active_theme].lowlight
        };
        scene.fill(
            Fill::NonZero,
            Affine::translate((0.0, translate_y)),
            color,
            None,
            &rect,
        );
    }
}

fn draw_grid(scene: &mut Scene, app: &AppState) -> bool {
    let (width, height) = app.window_size;
    let (grid_width, grid_height) = (
        width as f64 - 2.0 * GRID_PAD_X - BAR_WIDTH - BAR_PAD_X,
        height as f64 - 2.0 * GRID_PAD_Y,
    );
    let (card_width, card_height) = (
        (grid_width / app.col_num as f64) - (1.0 - 1.0 / app.col_num as f64) * GRID_GAP_X,
        (grid_height / app.row_num as f64) - (1.0 - 1.0 / app.row_num as f64) * GRID_GAP_Y,
    );
    let item_count = app.col_num * app.row_num;
    let current_page = app.active / item_count;
    let index_start = current_page * item_count;
    let index_end = (index_start + item_count).min(app.images.len());
    let card_rect = Rect::new(0.0, 0.0, card_width, card_height);
    let mut redraw = false;

    for i in 0..app.col_num {
        let translate_x = GRID_PAD_X + (card_width + GRID_GAP_X) * i as f64;
        for j in 0..app.row_num {
            let index = (j * app.col_num) + i + index_start;
            if index >= index_end {
                break;
            }
            let translate_y = GRID_PAD_Y + (card_height + GRID_GAP_Y) * j as f64;
            let translate = Affine::translate((translate_x, translate_y));
            let color = if index == app.active {
                Theme::ALL[app.active_theme].highlight
            } else {
                Theme::ALL[app.active_theme].lowlight
            };
            let image_state = get_image(&app.images[index], ImageSize::Thumbnail);
            match image_state {
                ImageState::Loading => {
                    let time = app.start_time.elapsed().as_secs_f64();
                    draw_border(scene, card_rect, translate, color);
                    draw_spinner(scene, card_rect, translate, time, color);
                    redraw = true;
                }
                ImageState::Loaded(ref image) => {
                    draw_image(scene, image, card_rect, translate, color);
                }
            }
        }
    }

    redraw
}

fn draw_border(scene: &mut Scene, rect: Rect, trans: Affine, color: Color) {
    let round_rect = rect.to_rounded_rect(7.0);
    let stroke = Stroke::new(CARD_THICK);
    scene.stroke(&stroke, trans, color, None, &round_rect);
}

fn draw_spinner(scene: &mut Scene, rect: Rect, trans: Affine, t: f64, color: Color) {
    let radius = 0.1 * rect.size().to_vec2().length();
    let spinner = CircleSegment::new(
        rect.center(),
        radius,
        radius * 0.95,
        2.0 * t,
        3.0 * PI / 2.0,
    );
    scene.fill(Fill::NonZero, trans, color, None, &spinner);
}

fn draw_image(scene: &mut Scene, image: &Image, rect: Rect, trans: Affine, color: Color) {
    let size = rect.size();
    let cw = size.width;
    let ch = size.height;
    let scale = (cw / image.width as f64).min(ch / image.height as f64);
    let sw = image.width as f64 * scale;
    let sh = image.height as f64 * scale;
    let round_rect = rect.to_rounded_rect(7.0);
    let border_rect = Rect::from_center_size(rect.center(), (sw, sh));
    scene.fill(
        Fill::NonZero,
        trans.then_translate(((cw - sw) / 2.0, (ch - sh) / 2.0).into()),
        image,
        Affine::scale(scale).into(),
        &round_rect,
    );
    draw_border(scene, border_rect, trans, color);
}
