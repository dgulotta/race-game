use crate::{direction::Direction, input::Action, tile::TileType};

use indexmap::IndexMap;
use notan::prelude::KeyCode;
use serde::{Deserialize, Deserializer, Serialize};
use serde_default::DefaultFromSerde;
use strum::IntoEnumIterator;

use super::graphics::TILE_SIZE;

type KeySettings = IndexMap<Action, KeyCode, hashbrown::DefaultHashBuilder>;

static NUM_KEYS: [KeyCode; 9] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Key5,
    KeyCode::Key6,
    KeyCode::Key7,
    KeyCode::Key8,
    KeyCode::Key9,
];

static KEYS_1: &[(Action, KeyCode)] = &[
    (Action::RotCW, KeyCode::R),
    (Action::RotCCW, KeyCode::E),
    (Action::Flip, KeyCode::F),
    (Action::ToggleLights, KeyCode::D),
    (Action::Reverse, KeyCode::S),
    (Action::SelectModify, KeyCode::Escape),
    (Action::SelectErase, KeyCode::Key0),
];

static KEYS_2: &[(Action, KeyCode)] = &[
    (Action::Scroll(Direction::Up), KeyCode::Up),
    (Action::Scroll(Direction::Down), KeyCode::Down),
    (Action::Scroll(Direction::Left), KeyCode::Left),
    (Action::Scroll(Direction::Right), KeyCode::Right),
    (Action::Undo, KeyCode::Z),
    (Action::Redo, KeyCode::L),
    (Action::Delete, KeyCode::Back),
    (Action::Start, KeyCode::C),
    (Action::StepBack, KeyCode::V),
    (Action::Pause, KeyCode::B),
    (Action::StepForward, KeyCode::N),
    (Action::Play, KeyCode::M),
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
}

impl Settings {
    pub fn tile_size(&self) -> f32 {
        TILE_SIZE * self.zoom.tile_size
    }
}
