//! The play alone screen: choose the level set and the lives.

use crate::solo::{Settings, LEVELS};
use macroquad::prelude::*;

const BG: Color = Color::new(0.0, 0.0, 0.08, 1.0);
const TEXT: Color = Color::new(1.0, 1.0, 0.63, 1.0);
const HIGHLIGHT: Color = Color::new(0.25, 0.25, 0.75, 1.0);
const GREEN: Color = Color::new(0.55, 1.0, 0.55, 1.0);

fn button(r: Rect, label: &str, size: f32, selected: bool) -> bool {
    let (mx, my) = mouse_position();
    let hover = r.contains(vec2(mx, my));
    if selected || hover {
        draw_rectangle(r.x, r.y, r.w, r.h, HIGHLIGHT);
    }
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, Color::new(0.5, 0.5, 0.9, 1.0));
    let m = measure_text(label, None, size as u16, 1.0);
    draw_text(label, r.x + (r.w - m.width) / 2.0, r.y + r.h / 2.0 + m.offset_y / 2.0, size, WHITE);
    hover && is_mouse_button_pressed(MouseButton::Left)
}

/// Show the screen until the player starts the game (Some) or quits (None).
pub async fn choose(initial: Settings) -> Option<Settings> {
    let mut choice = LEVELS.iter().position(|(f, _)| *f == initial.level).unwrap_or(0);
    let mut unlimited = initial.unlimited_lives;
    // 0..LEVELS.len(): a level; then the lives box; then Start
    let lives_row = LEVELS.len();
    let start_row = LEVELS.len() + 1;
    let mut focus = choice;
    loop {
        if is_quit_requested() || is_key_pressed(KeyCode::Escape) {
            return None;
        }
        let (sw, sh) = (screen_width(), screen_height());
        let scale = (sw / 640.0).min(sh / 480.0).max(0.6);
        let size = 22.0 * scale;
        let row = 30.0 * scale;
        let w = 440.0 * scale;
        let x = (sw - w) / 2.0;
        let mut y = (sh - row * (LEVELS.len() as f32 + 6.5)) / 2.0;
        clear_background(BG);

        let title = "Play alone";
        let m = measure_text(title, None, (size * 1.6) as u16, 1.0);
        draw_text(title, (sw - m.width) / 2.0, y + m.height, size * 1.6, GREEN);
        y += row * 1.4;
        let sub = "A game server will run on this computer, just for you.";
        let m = measure_text(sub, None, (size * 0.75) as u16, 1.0);
        draw_text(sub, (sw - m.width) / 2.0, y + m.height, size * 0.75, TEXT);
        y += row * 1.2;
        draw_text("Level set:", x, y + size, size, TEXT);
        y += row;

        for (i, (_, label)) in LEVELS.iter().enumerate() {
            let r = Rect::new(x, y, w, row * 0.9);
            if i == choice {
                draw_rectangle(r.x, r.y, r.w, r.h, HIGHLIGHT);
            }
            if i == focus {
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, WHITE);
            }
            let (mx, my) = mouse_position();
            if r.contains(vec2(mx, my)) && is_mouse_button_pressed(MouseButton::Left) {
                choice = i;
                focus = i;
            }
            let mark = if i == choice { "(o)" } else { "( )" };
            draw_text(&format!("{}  {}", mark, label), r.x + 8.0 * scale,
                      r.y + r.h * 0.72, size, WHITE);
            y += row;
        }

        y += row * 0.4;
        let r = Rect::new(x, y, w, row * 0.9);
        if focus == lives_row {
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, WHITE);
        }
        let (mx, my) = mouse_position();
        if r.contains(vec2(mx, my)) && is_mouse_button_pressed(MouseButton::Left) {
            unlimited = !unlimited;
            focus = lives_row;
        }
        let mark = if unlimited { "[x]" } else { "[ ]" };
        draw_text(&format!("{}  Unlimited lives", mark), r.x + 8.0 * scale,
                  r.y + r.h * 0.72, size, WHITE);
        y += row;
        let note = if unlimited {
            "Play for points, as long as you like."
        } else {
            "5 lives, and an extra life every 50000 points."
        };
        draw_text(note, x + 8.0 * scale, y + size * 0.8, size * 0.75, TEXT);
        y += row * 1.2;

        let br = Rect::new(x + w * 0.3, y, w * 0.4, row * 1.2);
        let clicked = button(br, "Start", size, focus == start_row);
        y += row * 1.6;
        let help = "Up / Down, Space, Enter - or the mouse.   Esc quits.";
        let m = measure_text(help, None, (size * 0.65) as u16, 1.0);
        draw_text(help, (sw - m.width) / 2.0, y + m.height, size * 0.65, TEXT);

        // keyboard
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::Tab) {
            focus = (focus + 1) % (start_row + 1);
            if focus < LEVELS.len() {
                choice = focus;
            }
        }
        if is_key_pressed(KeyCode::Up) {
            focus = (focus + start_row) % (start_row + 1);
            if focus < LEVELS.len() {
                choice = focus;
            }
        }
        if is_key_pressed(KeyCode::Space) && focus == lives_row {
            unlimited = !unlimited;
        }
        let enter = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
        if clicked || enter {
            return Some(Settings { level: LEVELS[choice].0.to_string(), unlimited_lives: unlimited });
        }
        next_frame().await;
    }
}

/// Show a message (e.g. "Starting...") for one frame.
pub async fn message(text: &str) {
    clear_background(BG);
    let (sw, sh) = (screen_width(), screen_height());
    let size = 26.0 * (sw / 640.0).min(sh / 480.0).max(0.6);
    let m = measure_text(text, None, size as u16, 1.0);
    draw_text(text, (sw - m.width) / 2.0, sh / 2.0, size, TEXT);
    next_frame().await;
}

/// Show an error until a key is pressed or the window is closed.
pub async fn error(text: &str) {
    loop {
        if is_quit_requested() || !get_keys_pressed().is_empty()
            || is_mouse_button_pressed(MouseButton::Left) {
            return;
        }
        clear_background(BG);
        let (sw, sh) = (screen_width(), screen_height());
        let size = 20.0 * (sw / 640.0).min(sh / 480.0).max(0.6);
        let mut y = sh / 3.0;
        for line in text.lines().chain(["", "Press a key to quit."].into_iter()) {
            let m = measure_text(line, None, size as u16, 1.0);
            draw_text(line, (sw - m.width) / 2.0, y, size, TEXT);
            y += size * 1.4;
        }
        next_frame().await;
    }
}
