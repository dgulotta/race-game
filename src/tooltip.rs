use crate::{
    course::{Course, TileCoord},
    direction::{DihedralElement, Direction, ROTATIONS},
    simulator::{is_entrance, is_exit, CarData, SimEvent, Simulator, SpawnPolicy},
    tile::{Tile, TileType},
};
use std::time::Duration;

pub struct TooltipState {
    pub tile: TileType,
    pub sim: Simulator,
    pub cars: Vec<CarData>,
    pub last_sim_time: Duration,
}

fn course_for(tile: TileType) -> Course {
    let mut course = Course::new_with_hasher_and_ptr_kind(Default::default());
    let orig = TileCoord(1, 1);
    course.insert_mut(
        orig,
        Tile {
            tile_type: tile,
            transform: DihedralElement::Id,
            offset: 0,
        },
    );
    for rot in ROTATIONS {
        let dir = rot * Direction::Up;
        if is_entrance(tile, dir) {
            course.insert_mut(
                orig - dir,
                Tile {
                    tile_type: TileType::Straight,
                    transform: rot,
                    offset: 0,
                },
            );
            course.insert_mut(
                orig - dir - dir,
                Tile {
                    tile_type: TileType::Finish,
                    transform: rot,
                    offset: 0,
                },
            );
        }
        if is_exit(tile, dir) {
            course.insert_mut(
                orig + dir,
                Tile {
                    tile_type: TileType::Straight,
                    transform: rot,
                    offset: 0,
                },
            );
        }
    }
    course
}

impl TooltipState {
    pub fn new(tile: TileType) -> Self {
        let mut sim = Simulator::new(course_for(tile), usize::MAX);
        let prob = 4;
        sim.set_spawn_policy(SpawnPolicy::Random(prob));
        Self {
            tile,
            sim,
            cars: Vec::new(),
            last_sim_time: Default::default(),
        }
    }

    pub fn advance(&mut self, time: Duration) {
        self.last_sim_time = time;
        self.sim.run_round();
        for ev in self.sim.events() {
            if let SimEvent::Round(v) = ev {
                self.cars = v;
            }
        }
    }
}
