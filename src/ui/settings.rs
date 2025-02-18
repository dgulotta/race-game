use crate::{direction::Direction, input::Action, tile::TileType};

use indexmap::IndexMap;
use notan::prelude::KeyCode;
use serde::{Deserialize, Deserializer, Serialize};

type KeySettings = IndexMap<Action, KeyCode, hashbrown::DefaultHashBuilder>;

pub fn default_key_settings() -> KeySettings {
    [
        (Action::RotCW, KeyCode::R),
        (Action::RotCCW, KeyCode::E),
        (Action::Flip, KeyCode::F),
        (Action::ToggleLights, KeyCode::D),
        (Action::SelectModify, KeyCode::Escape),
        (Action::SelectErase, KeyCode::Key0),
        (Action::SelectTile(TileType::Straight), KeyCode::Key1),
        (Action::SelectTile(TileType::Turn), KeyCode::Key2),
        (Action::SelectTile(TileType::Finish), KeyCode::Key3),
        (
            Action::SelectTile(TileType::LightIntersection),
            KeyCode::Key4,
        ),
        (
            Action::SelectTile(TileType::YieldIntersection),
            KeyCode::Key5,
        ),
        (Action::SelectTile(TileType::LightTurns), KeyCode::Key6),
        (Action::SelectTile(TileType::Merge), KeyCode::Key7),
        (
            Action::SelectTile(TileType::LightForwardTurn),
            KeyCode::Key8,
        ),
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
    ]
    .into_iter()
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

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(deserialize_with = "deserialize_keys")]
    pub keys: KeySettings,
    pub animate_tooltips: bool,
    pub tutorial: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            keys: default_key_settings(),
            animate_tooltips: true,
            tutorial: true,
        }
    }
}
