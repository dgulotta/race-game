use crate::{direction::Direction, input::Action, tile::TileType};

use indexmap::IndexMap;
use notan::{egui, prelude::KeyCode};
use serde::{Deserialize, Deserializer, Serialize};
use strum::IntoEnumIterator;

use super::graphics::TILE_SIZE;

type KeySettings = IndexMap<Action, KeyCode, hashbrown::DefaultHashBuilder>;

static NUM_KEYS: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

static KEYS_1: &[(Action, KeyCode)] = &[
    (Action::RotCW, KeyCode::KeyR),
    (Action::RotCCW, KeyCode::KeyE),
    (Action::Flip, KeyCode::KeyF),
    (Action::ToggleLights, KeyCode::KeyD),
    (Action::Reverse, KeyCode::KeyS),
    (Action::SelectModify, KeyCode::Escape),
    (Action::SelectErase, KeyCode::Digit0),
];

static KEYS_2: &[(Action, KeyCode)] = &[
    (Action::Scroll(Direction::Up), KeyCode::ArrowUp),
    (Action::Scroll(Direction::Down), KeyCode::ArrowDown),
    (Action::Scroll(Direction::Left), KeyCode::ArrowLeft),
    (Action::Scroll(Direction::Right), KeyCode::ArrowRight),
    (Action::Undo, KeyCode::KeyZ),
    (Action::Redo, KeyCode::KeyL),
    (Action::Delete, KeyCode::Backspace),
    (Action::Start, KeyCode::KeyC),
    (Action::StepBack, KeyCode::KeyV),
    (Action::Pause, KeyCode::KeyB),
    (Action::StepForward, KeyCode::KeyN),
    (Action::Play, KeyCode::KeyM),
    (Action::FastForward, KeyCode::Comma),
    (Action::End, KeyCode::Period),
];

pub fn default_key_settings() -> KeySettings {
    let tile_iter = TileType::iter().map(|t| (Action::SelectTile(t), NUM_KEYS[t as usize]));
    KEYS_1
        .iter()
        .copied()
        .chain(tile_iter)
        .chain(KEYS_2.iter().copied())
        .collect()
}

fn deserialize_keys<'de, D: Deserializer<'de>>(deserializer: D) -> Result<KeySettings, D::Error> {
    let mut settings = default_key_settings();
    let custom: KeySettings = Deserialize::deserialize(deserializer)?;
    for (k, v) in settings.iter_mut() {
        if let Some(vn) = custom.get(k) {
            *v = *vn;
        }
    }
    Ok(settings)
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default = "ZoomSettings::default")]
pub struct ZoomSettings {
    pub tile_size: f32,
    pub font_size: f32,
}

impl Default for ZoomSettings {
    fn default() -> Self {
        Self {
            tile_size: 1.0,
            font_size: 1.0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default = "Settings::default")]
pub struct Settings {
    #[serde(deserialize_with = "deserialize_keys")]
    pub keys: KeySettings,
    pub animate_tooltips: bool,
    pub tutorial: bool,
    pub smooth_animation: bool,
    pub zoom: ZoomSettings,
    pub ui_theme: egui::ThemePreference,
    pub bg_color: [f32; 3],
}

impl Settings {
    pub fn tile_size(&self) -> f32 {
        TILE_SIZE * self.zoom.tile_size
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            keys: default_key_settings(),
            animate_tooltips: true,
            tutorial: true,
            smooth_animation: true,
            zoom: Default::default(),
            ui_theme: egui::ThemePreference::System,
            bg_color: [1.0, 1.0, 1.0],
        }
    }
}
