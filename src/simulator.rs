use notan::random::rand::{Rng, thread_rng};

use crate::course::{Course, TileCoord};
use crate::direction::Direction;
use crate::tile::{Tile, TileType};
use core::ops::{Add, Sub};
use std::vec::Drain;

type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

static MAX_ROUNDS: usize = 1000;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct CarCoord(pub isize, pub isize);

impl Add<Direction> for CarCoord {
    type Output = Self;
    fn add(self, rhs: Direction) -> Self {
        Self(self.0 + rhs.dx(), self.1 + rhs.dy())
    }
}

impl Sub<Direction> for CarCoord {
    type Output = Self;
    fn sub(self, rhs: Direction) -> Self {
        Self(self.0 - rhs.dx(), self.1 - rhs.dy())
    }
}

impl From<TileCoord> for CarCoord {
    fn from(c: TileCoord) -> Self {
        Self(2 * c.0, 2 * c.1)
    }
}

impl From<CarCoord> for TileCoord {
    fn from(c: CarCoord) -> Self {
        assert!(c.0 % 2 == 0 && c.1 % 2 == 0);
        Self(c.0 / 2, c.1 / 2)
    }
}

impl CarCoord {
    pub fn add_multiple(self, dir: Direction, n: isize) -> Self {
        Self(self.0 + n * dir.dx(), self.1 + n * dir.dy())
    }

    pub fn distance_squared(self, other: CarCoord) -> isize {
        let dx = other.0 - self.0;
        let dy = other.1 - self.1;
        dx * dx + dy * dy
    }
}

pub enum SimEvent {
    Round(Vec<CarData>),
    Finished(usize),
    Crashed(usize),
}

pub enum SpawnPolicy {
    Always,
    Random(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CarData {
    pub id: usize,
    pub pos: CarCoord,
    pub dir: Direction,
}

impl CarData {
    fn tile_pos(&self) -> CarCoord {
        self.pos + self.dir
    }
}

enum CarStatus {
    Racing,
    Finished,
    Crashed,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum MoveStatus {
    Unknown,
    Moving,
    Stopped,
}

struct RoundRunner<'a> {
    sim: &'a mut Simulator,
    status: Vec<MoveStatus>,
    car_grid: HashMap<CarCoord, usize>,
    cars_new: Vec<CarData>,
}

pub struct Simulator {
    course: Course,
    starts: Vec<(CarCoord, Direction)>,
    round: usize,
    cars: Vec<CarData>,
    spawn_policy: SpawnPolicy,
    next_car: usize,
    max_cars: usize,
    stream: Vec<SimEvent>,
}

fn random_8(n: u8) -> bool {
    thread_rng().gen_ratio(n as u32, 8)
}

pub const fn is_entrance(tile: TileType, car_dir: Direction) -> bool {
    match tile {
        TileType::Straight
        | TileType::Turn
        | TileType::Finish
        | TileType::LightForwardTurn
        | TileType::LightTurns => matches!(car_dir, Direction::Up),
        TileType::Merge | TileType::YieldIntersection | TileType::LightIntersection => {
            matches!(car_dir, Direction::Up | Direction::Left)
        }
    }
}

pub const fn is_exit(tile: TileType, car_dir: Direction) -> bool {
    match tile {
        TileType::Straight | TileType::Finish | TileType::Merge => matches!(car_dir, Direction::Up),
        TileType::Turn => matches!(car_dir, Direction::Left),
        TileType::LightTurns => matches!(car_dir, Direction::Left | Direction::Right),
        TileType::LightForwardTurn | TileType::LightIntersection | TileType::YieldIntersection => {
            matches!(car_dir, Direction::Up | Direction::Left)
        }
    }
}

impl<'a> RoundRunner<'a> {
    fn new(sim: &'a mut Simulator) -> Self {
        let car_grid: HashMap<CarCoord, usize> = sim
            .cars
            .iter()
            .enumerate()
            .map(|(i, c)| (c.pos, i))
            .collect();
        let status = vec![MoveStatus::Unknown; sim.cars.len()];
        Self {
            sim,
            status,
            car_grid,
            cars_new: Vec::new(),
        }
    }

    fn car_at(&self, pos: CarCoord) -> Option<&CarData> {
        self.car_grid.get(&pos).map(|x| &self.sim.cars[*x])
    }

    fn is_entering_tile(&self, tile_pos: CarCoord, from_dir: Direction) -> bool {
        match self.car_at(tile_pos - from_dir) {
            Some(c) => c.dir == from_dir,
            None => false,
        }
    }

    fn is_blocked_incoming(&self, pos: CarCoord, dir: Direction) -> bool {
        if let Some(tile) = self.sim.tile_at(pos) {
            let dir_norm = tile.transform.apply_inverse(dir);
            match tile.tile_type {
                TileType::LightIntersection => {
                    ((dir_norm as usize) ^ self.sim.round ^ (tile.offset as usize) ^ 1) & 1 != 0
                }
                TileType::YieldIntersection => {
                    dir_norm == Direction::Up
                        && self.is_entering_tile(pos, tile.transform * Direction::Left)
                }
                TileType::Merge => {
                    dir_norm == Direction::Left
                        && self.is_entering_tile(pos, tile.transform * Direction::Up)
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn check_blocked_outgoing(&mut self, tile_pos: CarCoord, from_dir: Direction) -> bool {
        let new_pos = self.sim.out_pos(tile_pos, from_dir);
        self.car_grid
            .get(&new_pos)
            .copied()
            .map(|n| !self.try_move(n))
            .unwrap_or(false)
    }

    fn check_move(&mut self, car: &CarData) -> bool {
        let tile_pos = car.tile_pos();
        !(self.is_blocked_incoming(tile_pos, car.dir)
            || self.check_blocked_outgoing(tile_pos, car.dir))
    }

    fn try_move(&mut self, id: usize) -> bool {
        match self.status[id] {
            MoveStatus::Moving => true,
            MoveStatus::Stopped => false,
            MoveStatus::Unknown => {
                self.status[id] = MoveStatus::Moving;
                let car = self.sim.cars[id];
                let success = self.check_move(&car);
                if success {
                    self.cars_new.push(self.sim.new_pos(&car));
                } else {
                    self.status[id] = MoveStatus::Stopped;
                    self.cars_new.push(car);
                }
                success
            }
        }
    }

    fn move_cars(&mut self) {
        for i in 0..self.sim.cars.len() {
            self.try_move(i);
        }
    }

    fn is_spot_free(&self, pos: CarCoord) -> bool {
        self.car_grid
            .get(&pos)
            .is_none_or(|&i| self.status[i] == MoveStatus::Moving)
    }

    fn check_add_car(&self) -> bool {
        match self.sim.spawn_policy {
            SpawnPolicy::Always => true,
            SpawnPolicy::Random(n) => random_8(n),
        }
    }

    fn add_cars(&mut self) {
        for start in self.sim.starts.iter() {
            if self.sim.next_car < self.sim.max_cars
                && self.is_spot_free(start.0)
                && self.check_add_car()
            {
                self.cars_new.push(CarData {
                    id: self.sim.next_car,
                    pos: start.0,
                    dir: start.1,
                });
                self.sim.next_car += 1;
            }
        }
    }

    fn remove_finished(&mut self) {
        self.sim.cars.clear();
        for car in self.cars_new.iter() {
            match self.sim.check_finish(car.pos, car.dir) {
                CarStatus::Racing => self.sim.cars.push(*car),
                CarStatus::Finished => self.sim.stream.push(SimEvent::Finished(car.id)),
                CarStatus::Crashed => self.sim.stream.push(SimEvent::Crashed(car.id)),
            }
        }
    }

    fn send_event(&mut self, event: SimEvent) {
        self.sim.stream.push(event)
    }
}

impl Simulator {
    pub fn new(course: Course, max_cars: usize) -> Self {
        let starts: Vec<_> = course
            .iter()
            .filter_map(|(pos, tile)| {
                if tile.tile_type == TileType::Finish {
                    let dir = tile.transform * Direction::Up;
                    Some((CarCoord::from(*pos) + dir, dir))
                } else {
                    None
                }
            })
            .collect();
        Self {
            course,
            starts,
            round: 0,
            cars: Vec::new(),
            spawn_policy: SpawnPolicy::Always,
            next_car: 0,
            max_cars,
            stream: Vec::new(),
        }
    }

    pub fn events(&mut self) -> Drain<'_, SimEvent> {
        self.stream.drain(..)
    }

    pub fn set_spawn_policy(&mut self, p: SpawnPolicy) {
        self.spawn_policy = p;
    }

    fn out_dir(&self, tile: &Tile, in_dir: Direction) -> Direction {
        let offset = ((self.round as u8) ^ tile.offset) & 1 != 0;
        match tile.tile_type {
            TileType::Straight | TileType::LightIntersection | TileType::YieldIntersection => {
                in_dir
            }
            TileType::Turn => tile.transform * Direction::Left,
            TileType::LightTurns => {
                tile.transform
                    * if offset {
                        Direction::Right
                    } else {
                        Direction::Left
                    }
            }
            TileType::LightForwardTurn => {
                tile.transform
                    * if offset {
                        Direction::Up
                    } else {
                        Direction::Left
                    }
            }
            TileType::Merge => tile.transform * Direction::Up,
            TileType::Finish => unreachable!(),
        }
    }

    fn out_pos(&self, tile_pos: CarCoord, in_dir: Direction) -> CarCoord {
        tile_pos + self.out_dir(self.tile_at(tile_pos).unwrap(), in_dir)
    }

    fn tile_at(&self, pos: CarCoord) -> Option<&Tile> {
        self.course.get(&pos.into())
    }

    fn new_pos(&self, car: &CarData) -> CarData {
        let tile_pos = car.tile_pos();
        let tile = self.tile_at(tile_pos).unwrap();
        let new_dir = self.out_dir(tile, car.dir);
        CarData {
            id: car.id,
            pos: tile_pos + new_dir,
            dir: new_dir,
        }
    }

    fn check_finish(&self, pos: CarCoord, car_dir: Direction) -> CarStatus {
        if let Some(tile) = self.tile_at(pos + car_dir) {
            let car_dir_norm = tile.transform.apply_inverse(car_dir);
            if is_entrance(tile.tile_type, car_dir_norm) {
                if tile.tile_type == TileType::Finish {
                    CarStatus::Finished
                } else {
                    CarStatus::Racing
                }
            } else {
                CarStatus::Crashed
            }
        } else {
            CarStatus::Crashed
        }
    }

    pub fn run_round(&mut self) {
        let mut runner = RoundRunner::new(self);
        runner.move_cars();
        runner.add_cars();
        runner.send_event(SimEvent::Round(runner.cars_new.clone()));
        runner.remove_finished();
        self.round += 1;
    }

    pub fn get_course(&self) -> &Course {
        &self.course
    }

    pub fn take_course(self) -> Course {
        self.course
    }

    pub fn get_round(&self) -> usize {
        self.round
    }

    pub fn is_finished(&self) -> bool {
        self.round >= MAX_ROUNDS || (self.cars.is_empty() && self.next_car >= self.max_cars)
    }

    pub fn get_cars(&self) -> &Vec<CarData> {
        &self.cars
    }
}
