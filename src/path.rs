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

struct PathIter {
    start: CarCoord,
    end: CarCoord,
}

impl Iterator for PathIter {
    type Item = CarCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            let dx = self.end.0 - self.start.0;
            let dy = self.end.1 - self.start.1;
            self.start = if self.start.0 & 1 == 0 {
                match dy {
                    ..=-2 => CarCoord(self.start.0, self.start.1 - 2),
                    -1 | 1 => CarCoord(self.start.0 + dx.signum(), self.end.1),
                    0 => return None,
                    2.. => CarCoord(self.start.0, self.start.1 + 2),
                }
            } else {
                match dx {
                    ..=-2 => CarCoord(self.start.0 - 2, self.start.1),
                    -1 | 1 => CarCoord(self.end.0, self.start.1 + dy.signum()),
                    0 => return None,
                    2.. => CarCoord(self.start.0 + 2, self.start.1),
                }
            };
            Some(self.start)
        }
    }
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
        if let Some(old_last) = self.0.last().copied() {
            let it = PathIter {
                start: old_last,
                end: pos,
            };
            for new in it {
                if let Some(pvs) = self.second_last() {
                    let last = *self.0.last().unwrap();
                    if pvs == new {
                        self.0.pop();
                    } else if common_tile(pvs, last) == common_tile(last, new) {
                        self.0.pop();
                        self.0.push(new);
                    } else {
                        self.0.push(new);
                    }
                } else {
                    self.0.push(new);
                }
            }
        } else {
            self.0.push(pos);
        }
    }
}
