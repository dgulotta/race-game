use crate::{direction::Direction, input::Action, tile::TileType};

use indexmap::IndexMap;
use notan::{egui, prelude::KeyCode};
use serde::{Deserialize, Deserializer, Serialize};
use serde_default::DefaultFromSerde;
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

const fn one() -> f32 {
    1.0
}
const fn true_fn() -> bool {
    true
}

const fn default_theme() -> egui::ThemePreference {
    egui::ThemePreference::System
}

const fn default_bg_color() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

#[derive(Clone, Serialize, Deserialize, DefaultFromSerde)]
pub struct ZoomSettings {
    #[serde(default = "one")]
    pub tile_size: f32,
    #[serde(default = "one")]
    pub font_size: f32,
}

#[derive(Clone, Serialize, Deserialize, DefaultFromSerde)]
pub struct Settings {
    #[serde(
        default = "default_key_settings",
        deserialize_with = "deserialize_keys"
    )]
    pub keys: KeySettings,
    #[serde(default = "true_fn")]
    pub animate_tooltips: bool,
    #[serde(default = "true_fn")]
    pub tutorial: bool,
    #[serde(default = "true_fn")]
    pub smooth_animation: bool,
    #[serde(default)]
    pub zoom: ZoomSettings,
    #[serde(default = "default_theme")]
    pub ui_theme: egui::ThemePreference,
    #[serde(default = "default_bg_color")]
    pub bg_color: [f32; 3],
}

impl Settings {
    pub fn tile_size(&self) -> f32 {
        TILE_SIZE * self.zoom.tile_size
    }
}
