use crate::{course::TileCoord, input::Action};

use super::{graphics::TILE_SIZE, settings::Settings};
use notan::{
    math::Vec2,
    prelude::{App, KeyCode},
};

pub fn mouse_coords(app: &App, offset: &Vec2) -> TileCoord {
    let x = ((app.mouse.x - offset.x) / TILE_SIZE).floor() as isize;
    let y = ((app.mouse.y - offset.y) / TILE_SIZE).floor() as isize;
    TileCoord(x, y)
}

pub fn check_key_press(app: &App, settings: &Settings, key: Action) -> bool {
    app.keyboard.was_pressed(settings.keys[&key])
}

pub fn key_name(code: KeyCode) -> String {
    let n = code as u32;
    match code {
        _ if (0..=8).contains(&n) => (n + 1).to_string(),
        KeyCode::Key0 => 0.to_string(),
        KeyCode::Back => "Backspace".to_string(),
        KeyCode::Comma => ",".to_string(),
        KeyCode::Period => ".".to_string(),
        KeyCode::Escape => "Esc".to_string(),
        _ => format!("{:?}", code),
    }
}
