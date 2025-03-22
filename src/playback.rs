use std::time::Duration;

use notan::math::Vec2;

use crate::{
    direction::Direction,
    simulator::{CarCoord, CarData},
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Playback {
    Paused,
    Playing,
    Fast,
}

impl Playback {
    pub const fn frame_duration(self) -> Duration {
        match self {
            Self::Paused => Duration::from_millis(100),
            Self::Playing => Duration::from_millis(500),
            Self::Fast => Duration::from_millis(100),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CarPosition {
    pub pos: CarCoord,
    pub dir: Direction,
}

pub struct CarAnimation {
    pub id: usize,
    pub old_pos: Option<CarPosition>,
    pub new_pos: CarPosition,
}

pub struct AnimationData {
    pub animations: Vec<CarAnimation>,
    pub start: Duration,
}

pub struct CarPosF {
    pub pos: Vec2,
    pub dir: Vec2,
}

impl From<CarData> for CarPosition {
    fn from(value: CarData) -> Self {
        Self {
            pos: value.pos,
            dir: value.dir,
        }
    }
}

impl From<CarPosition> for CarPosF {
    fn from(value: CarPosition) -> Self {
        Self {
            pos: Vec2::new(value.pos.0 as f32, value.pos.1 as f32),
            dir: Vec2::new(value.dir.dx() as f32, value.dir.dy() as f32),
        }
    }
}

impl From<CarData> for CarPosF {
    fn from(value: CarData) -> Self {
        let pos: CarPosition = value.into();
        pos.into()
    }
}

impl CarAnimation {
    pub fn position_at_time(&self, time: f32) -> CarPosF {
        match &self.old_pos {
            Some(old) if old != &self.new_pos => {
                let t = time.clamp(0.0, 1.0);
                let tc = 1.0 - t;
                let p0 = Self::car_pos_vec(old.pos);
                let p1 = Self::car_pos_vec(old.pos.add_multiple(old.dir, 1));
                let p2 = Self::car_pos_vec(self.new_pos.pos.add_multiple(self.new_pos.dir, -1));
                let p3 = Self::car_pos_vec(self.new_pos.pos);
                let p = tc * tc * tc * p0 + 3.0 * tc * t * (tc * p1 + t * p2) + t * t * t * p3;
                let v = 3.0
                    * (-tc * tc * p0 + tc * (tc - 2.0 * t) * p1 - t * (t - 2.0 * tc) * p2
                        + t * t * p3);
                CarPosF {
                    pos: p,
                    dir: v.normalize(),
                }
            }
            _ => self.new_pos.into(),
        }
    }

    fn car_pos_vec(pos: CarCoord) -> Vec2 {
        Vec2::new(pos.0 as f32, pos.1 as f32)
    }
}

pub fn animations(old: &[CarData], new: &[CarData]) -> Vec<CarAnimation> {
    let mut cars: hashbrown::HashMap<_, _> = new
        .iter()
        .map(|c| {
            (
                c.id,
                CarAnimation {
                    id: c.id,
                    old_pos: None,
                    new_pos: CarPosition {
                        pos: c.pos,
                        dir: c.dir,
                    },
                },
            )
        })
        .collect();
    for c in old.iter() {
        if let Some(p) = cars.get_mut(&c.id) {
            p.old_pos = Some(CarPosition {
                pos: c.pos,
                dir: c.dir,
            });
        }
    }
    cars.into_values().collect()
}
