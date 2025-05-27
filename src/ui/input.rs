use crate::{course::TileCoord, input::Action, simulator::CarCoord};

use super::settings::Settings;
use notan::{
    math::Vec2,
    prelude::{App, KeyCode},
};

pub fn mouse_coords(app: &App, settings: &Settings, offset: &Vec2) -> TileCoord {
    let tsz = settings.tile_size();
    let x = ((app.mouse.x - offset.x) / tsz).floor() as isize;
    let y = ((app.mouse.y - offset.y) / tsz).floor() as isize;
    TileCoord(x, y)
}

pub fn mouse_coords_car(app: &App, settings: &Settings, offset: &Vec2) -> Option<CarCoord> {
    let tsz = settings.tile_size();
    let xf = (app.mouse.x - offset.x) / tsz;
    let yf = (app.mouse.y - offset.y) / tsz;
    if (0.25..0.75).contains(&xf.fract()) && (0.25..0.75).contains(&yf.fract()) {
        None
    } else {
        let s = (xf + yf).floor() as isize;
        let d = (xf - yf).floor() as isize;
        Some(CarCoord(s + d, s - d - 1))
    }
}

pub fn check_key_press(app: &App, settings: &Settings, key: Action) -> bool {
    app.keyboard.was_pressed(settings.keys[&key])
}

pub fn key_name(code: KeyCode) -> String {
    match code {
        KeyCode::Escape => "Esc".to_string(),
        KeyCode::Comma => ",".to_string(),
        KeyCode::Period => ".".to_string(),
        _ => {
            let name = format!("{:?}", code);
            name.trim_start_matches("Arrow")
                .trim_start_matches("Digit")
                .trim_start_matches("Key")
                .to_string()
        }
    }
}
