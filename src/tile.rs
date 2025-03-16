use crate::direction::DihedralElement;
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, FromRepr};

#[derive(
    Clone, Copy, PartialEq, Eq, FromRepr, Serialize, Deserialize, Enum, EnumIter, Debug, Hash,
)]
pub enum TileType {
    Straight,
    Turn,
    Finish,
    LightTurns,
    LightForwardTurn,
    Merge,
    YieldIntersection,
    LightIntersection,
}

impl TileType {
    pub const fn has_lights(self) -> bool {
        matches!(
            self,
            Self::LightIntersection | Self::LightTurns | Self::LightForwardTurn
        )
    }

    pub const fn name(self) -> &'static str {
        use TileType::*;
        match self {
            Straight => "Straight",
            Turn => "Turn",
            Finish => "Start/Finish",
            LightIntersection => "Intersection with lights",
            YieldIntersection => "Intersection with yield sign",
            LightTurns => "Left/right turn with lights",
            Merge => "Merge with yield sign",
            LightForwardTurn => "Straight/turn with lights",
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tile {
    pub tile_type: TileType,
    pub transform: DihedralElement,
    pub offset: u8,
}

impl Tile {
    pub fn apply_transform(self, transform: DihedralElement, offset: u8) -> Self {
        Self {
            transform: transform * self.transform,
            offset: offset ^ self.offset,
            ..self
        }
    }
    pub fn default_for_type(tile_type: TileType) -> Self {
        Self {
            tile_type,
            transform: DihedralElement::Id,
            offset: 0,
        }
    }
}
