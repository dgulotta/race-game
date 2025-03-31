use crate::{
    course::TileCoord,
    direction::{DihedralElement, Direction, rotation_for},
    simulator::CarCoord,
    tile::{Tile, TileType},
};

#[derive(Default)]
pub struct Path(Vec<CarCoord>);

fn div_4_round(x: isize) -> Option<isize> {
    if x.rem_euclid(4) == 2 {
        None
    } else {
        Some((x + 1).div_euclid(4))
    }
}

fn is_boundary(t: TileCoord, p: CarCoord) -> bool {
    let c: CarCoord = t.into();
    p.distance_squared(c) == 1
}

fn common_tile(p1: CarCoord, p2: CarCoord) -> Option<TileCoord> {
    let x = p1.0 + p2.0;
    let y = p1.1 + p2.1;
    let t = TileCoord(div_4_round(x)?, div_4_round(y)?);
    if is_boundary(t, p1) && is_boundary(t, p2) {
        Some(t)
    } else {
        None
    }
}

fn direction_to(p1: CarCoord, p2: CarCoord) -> Direction {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    if dx > 0 {
        Direction::Right
    } else if dx < 0 {
        Direction::Left
    } else if dy > 0 {
        Direction::Down
    } else {
        Direction::Up
    }
}

pub fn track_tile(p1: CarCoord, p2: CarCoord) -> (TileCoord, Tile) {
    let pos = common_tile(p1, p2).unwrap();
    let car_pos: CarCoord = pos.into();
    let d1 = direction_to(car_pos, p1);
    let d2 = direction_to(car_pos, p2);
    let tile = if d2 == d1.opposite() {
        Tile {
            tile_type: TileType::Straight,
            transform: rotation_for(Direction::Up, d2),
            offset: 0,
        }
    } else {
        let transform = if d2 == DihedralElement::Rot90 * d1 {
            rotation_for(Direction::Left, d2)
        } else {
            rotation_for(Direction::Left, d2) * DihedralElement::Flip90
        };
        Tile {
            tile_type: TileType::Turn,
            transform,
            offset: 0,
        }
    };
    (pos, tile)
}

impl Path {
    pub fn path(&self) -> &[CarCoord] {
        &self.0
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    fn second_last(&self) -> Option<CarCoord> {
        if self.0.len() >= 2 {
            Some(self.0[self.0.len() - 2])
        } else {
            None
        }
    }

    pub fn add(&mut self, pos: CarCoord) {
        if let Some(last) = self.0.last().copied() {
            if let Some(t1) = common_tile(last, pos) {
                if let Some(pvs) = self.second_last() {
                    if pvs == pos {
                        self.0.pop();
                    } else if common_tile(pvs, last) == Some(t1) {
                        self.0.pop();
                        self.0.push(pos);
                    } else {
                        self.0.push(pos);
                    }
                } else {
                    self.0.push(pos);
                }
            }
        } else {
            self.0.push(pos);
        }
    }
}
