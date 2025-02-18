use core::ops::Mul;
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, FromRepr, VariantNames};

use crate::course::TileCoord;

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    FromRepr,
    EnumIter,
    Debug,
    Serialize,
    Deserialize,
    Enum,
    VariantNames,
    Hash,
)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub const fn dx(self) -> isize {
        match self {
            Self::Left => -1,
            Self::Right => 1,
            _ => 0,
        }
    }

    pub const fn dy(self) -> isize {
        match self {
            Self::Up => -1,
            Self::Down => 1,
            _ => 0,
        }
    }

    pub const fn opposite(self) -> Self {
        Self::from_repr((self as usize) ^ 2).unwrap()
    }

    pub const fn name(self) -> &'static str {
        Self::VARIANTS[self as usize]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, FromRepr, Debug, Serialize, Deserialize, EnumIter)]
pub enum DihedralElement {
    Id,
    Rot90,
    Rot180,
    Rot270,
    Flip0,
    Flip45,
    Flip90,
    Flip135,
}

impl DihedralElement {
    pub const fn sign(self) -> isize {
        match self {
            Self::Flip0 | Self::Flip45 | Self::Flip90 | Self::Flip135 => -1,
            _ => 1,
        }
    }

    const fn shift(self) -> isize {
        (self as isize) & 3
    }

    pub const fn apply(self, rhs: Direction) -> Direction {
        let i = (self.sign() * (rhs as isize) + self.shift()) & 3;
        Direction::from_repr(i as usize).unwrap()
    }

    pub const fn apply_inverse(self, rhs: Direction) -> Direction {
        let i = (self.sign() * (rhs as isize - self.shift())) & 3;
        Direction::from_repr(i as usize).unwrap()
    }

    pub const fn inverse(self) -> Self {
        match self {
            Self::Rot90 => Self::Rot270,
            Self::Rot270 => Self::Rot90,
            _ => self,
        }
    }
}

pub static ROTATIONS: [DihedralElement; 4] = [
    DihedralElement::Id,
    DihedralElement::Rot90,
    DihedralElement::Rot180,
    DihedralElement::Rot270,
];

impl Mul<Direction> for DihedralElement {
    type Output = Direction;
    fn mul(self, rhs: Direction) -> Direction {
        self.apply(rhs)
    }
}

impl Mul<Self> for DihedralElement {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let s = ((self as usize) ^ (rhs as usize)) & 4;
        let i = (self.shift() + self.sign() * rhs.shift()) & 3;
        Self::from_repr(s | (i as usize)).unwrap()
    }
}

impl Mul<TileCoord> for DihedralElement {
    type Output = TileCoord;
    fn mul(self, rhs: TileCoord) -> TileCoord {
        let r = self * Direction::Right;
        let d = self * Direction::Down;
        TileCoord(
            r.dx() * rhs.0 + d.dx() * rhs.1,
            r.dy() * rhs.0 + d.dy() * rhs.1,
        )
    }
}

pub const fn rotation_for(from: Direction, to: Direction) -> DihedralElement {
    let n = ((to as isize) - (from as isize)) & 3;
    DihedralElement::from_repr(n as usize).unwrap()
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Isometry {
    pub dihedral: DihedralElement,
    pub offset: TileCoord,
}

impl Mul<TileCoord> for Isometry {
    type Output = TileCoord;
    fn mul(self, rhs: TileCoord) -> TileCoord {
        self.offset + self.dihedral * rhs
    }
}

impl Isometry {
    pub fn inverse(&self) -> Self {
        Self {
            dihedral: self.dihedral.inverse(),
            offset: -(self.dihedral.inverse() * self.offset),
        }
    }

    pub fn apply_inverse(&self, pos: TileCoord) -> TileCoord {
        self.dihedral.inverse() * (pos - self.offset)
    }

    pub fn from_anchors(from: TileCoord, to: TileCoord, dihedral: DihedralElement) -> Self {
        let offset = to - dihedral * from;
        Self { dihedral, offset }
    }
}
