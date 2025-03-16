use std::hash::Hash;

use enum_map::EnumMap;
use serde::{Deserialize, Deserializer, Serialize};

use crate::tile::TileType;

fn deserialize_banned<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<EnumMap<TileType, bool>, D::Error> {
    let ban_list: Vec<TileType> = Deserialize::deserialize(deserializer)?;
    let mut ban_map: EnumMap<TileType, bool> = Default::default();
    for t in ban_list {
        ban_map[t] = true;
    }
    Ok(ban_map)
}

#[derive(Deserialize, Clone)]
pub struct LevelData {
    pub name: String,
    pub cars: usize,
    pub finish: Vec<usize>,
    pub tutorial: Option<usize>,
    #[serde(default, deserialize_with = "deserialize_banned")]
    pub banned: EnumMap<TileType, bool>,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct SolveData {
    pub tiles: usize,
    pub turns: usize,
}

impl SolveData {
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            tiles: self.tiles.min(other.tiles),
            turns: self.turns.min(other.turns),
        }
    }
    pub fn combine_option(&self, other: &Option<Self>) -> Self {
        match other {
            Some(d) => self.combine(d),
            None => *self,
        }
    }
}

impl Hash for LevelData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cars.hash(state);
        self.finish.hash(state);
    }
}
