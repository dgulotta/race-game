use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct LevelData {
    pub name: String,
    pub cars: usize,
    pub finish: Vec<usize>,
    pub tutorial: Option<usize>,
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
