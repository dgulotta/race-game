use notan::math::Vec2;
use serde::{Deserialize, Serialize};
use takeable::Takeable;

use crate::direction::{DihedralElement, Direction};
use crate::level::LevelData;
use crate::save::save_course;
use crate::tile::{Tile, TileType};
use std::borrow::Borrow;
use std::ops::{Add, Neg, RangeInclusive, Sub};
use std::rc::Rc;

pub type Course = rpds::HashTrieMap<
    TileCoord,
    Tile,
    archery::shared_pointer::kind::RcK,
    rustc_hash::FxBuildHasher,
>;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TileCoord(pub isize, pub isize);

impl Add<Direction> for TileCoord {
    type Output = Self;
    fn add(self, rhs: Direction) -> Self {
        Self(self.0 + rhs.dx(), self.1 + rhs.dy())
    }
}

impl Sub<Direction> for TileCoord {
    type Output = Self;
    fn sub(self, rhs: Direction) -> Self {
        Self(self.0 - rhs.dx(), self.1 - rhs.dy())
    }
}

impl Sub<Self> for TileCoord {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        TileCoord(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Add<Self> for TileCoord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl From<Direction> for TileCoord {
    fn from(value: Direction) -> Self {
        Self(value.dx(), value.dy())
    }
}

impl Neg for TileCoord {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0, -self.1)
    }
}

pub fn bounding_rect<I>(squares: I) -> (RangeInclusive<isize>, RangeInclusive<isize>)
where
    I: IntoIterator,
    I::Item: Borrow<TileCoord>,
{
    let mut xmin = isize::MAX;
    let mut xmax = isize::MIN;
    let mut ymin = isize::MAX;
    let mut ymax = isize::MIN;
    for p in squares {
        let pos = p.borrow();
        if pos.0 < xmin {
            xmin = pos.0;
        }
        if pos.0 > xmax {
            xmax = pos.0;
        }
        if pos.1 < ymin {
            ymin = pos.1;
        }
        if pos.1 > ymax {
            ymax = pos.1;
        }
    }
    (xmin..=xmax, ymin..=ymax)
}

#[derive(Clone)]
pub struct CourseEditState {
    course: Course,
    finish: Option<TileCoord>,
}

impl CourseEditState {
    fn remove(&mut self, pos: TileCoord) -> bool {
        if self.course.contains_key(&pos) {
            if self.finish == Some(pos) {
                self.finish = None;
            }
            self.course.remove_mut(&pos);
            true
        } else {
            false
        }
    }

    fn set(&mut self, pos: TileCoord, tile: Tile) -> bool {
        if self.course.get(&pos) == Some(&tile) {
            false
        } else {
            self.course.insert_mut(pos, tile);
            if tile.tile_type == TileType::Finish {
                if let Some(old_finish) = self.finish {
                    self.course.get_mut(&old_finish).unwrap().tile_type = TileType::Straight;
                }
                self.finish = Some(pos);
            } else if self.finish == Some(pos) {
                self.finish = None;
            }
            true
        }
    }
    pub fn from_course(course: Course) -> Self {
        let finish = course.iter().find_map(|(pos, tile)| {
            if tile.tile_type == TileType::Finish {
                Some(*pos)
            } else {
                None
            }
        });
        Self { finish, course }
    }
}

pub struct CourseEdit {
    stack: Vec<CourseEditState>,
    pos: usize,
    level: Rc<LevelData>,
}

pub struct Transaction<'a> {
    edit: &'a mut CourseEdit,
    state: Takeable<CourseEditState>,
    changed: bool,
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if self.changed {
            self.edit.push(self.state.take());
        }
    }
}

impl<'a> Transaction<'a> {
    fn new(edit: &'a mut CourseEdit) -> Self {
        let state = edit.get_state().clone();
        Self {
            edit,
            state: Takeable::new(state),
            changed: false,
        }
    }
    fn course(&self) -> &Course {
        &self.state.course
    }
    fn course_mut(&mut self) -> &mut Course {
        &mut self.state.course
    }
    pub fn remove(&mut self, pos: TileCoord) {
        self.changed |= self.state.remove(pos);
    }
    pub fn set(&mut self, pos: TileCoord, tile: Tile) {
        self.changed |= self.state.set(pos, tile);
    }
    pub fn toggle_lights(&mut self, pos: TileCoord) {
        if let Some(tile) = self.course().get(&pos) {
            if tile.tile_type.has_lights() {
                let new_tile = Tile {
                    offset: tile.offset ^ 1,
                    ..*tile
                };
                self.course_mut().insert_mut(pos, new_tile);
                self.changed = true;
            }
        }
    }
    pub fn apply_transform(&mut self, pos: TileCoord, trans: DihedralElement) {
        if let Some(tile) = self.course_mut().get_mut(&pos) {
            tile.transform = trans * tile.transform;
            self.changed = true;
        }
    }
    pub fn commit(self) {}
}

impl CourseEdit {
    pub fn new(course: Course, level: Rc<LevelData>) -> Self {
        let state = CourseEditState::from_course(course);
        Self {
            stack: vec![state],
            pos: 0,
            level,
        }
    }
    fn get_state(&self) -> &CourseEditState {
        &self.stack[self.pos]
    }
    pub fn get_course(&self) -> &Course {
        &self.get_state().course
    }
    pub fn take_course(mut self) -> Course {
        self.stack.pop().unwrap().course
    }
    pub fn get_finish(&self) -> Option<TileCoord> {
        self.get_state().finish
    }
    pub fn get(&self, c: TileCoord) -> Option<&Tile> {
        self.get_course().get(&c)
    }
    pub fn edit(&mut self) -> Transaction {
        Transaction::new(self)
    }
    pub fn set_single(&mut self, pos: TileCoord, tile: Tile) {
        self.edit().set(pos, tile);
    }
    pub fn set_course(&mut self, course: Course) {
        self.push(CourseEditState::from_course(course));
    }
    pub fn save(&self) {
        save_course(&self.level, self.get_course());
    }
    fn push(&mut self, st: CourseEditState) {
        self.stack.truncate(self.pos + 1);
        self.stack.push(st);
        self.pos += 1;
        self.save();
    }
    pub fn undo(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
            self.save();
        }
    }
    pub fn redo(&mut self) {
        if self.pos < self.stack.len() - 1 {
            self.pos += 1;
            self.save();
        }
    }
}

pub fn course_center(course: &Course) -> Vec2 {
    if course.is_empty() {
        Vec2::default()
    } else {
        let num: Vec2 = course
            .keys()
            .map(|k| Vec2::new(k.0 as f32, k.1 as f32))
            .sum();
        num / (course.size() as f32)
    }
}
