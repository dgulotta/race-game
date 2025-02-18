use std::ops::RangeInclusive;

use notan::log::warn;
use rustc_hash::FxHashSet;

use crate::{
    course::{bounding_rect, course_center, Course, CourseEdit, TileCoord},
    direction::{DihedralElement, Isometry},
    tile::Tile,
};

#[derive(Default)]
pub struct SelectState {
    pub selection: FxHashSet<TileCoord>,
    pub drag: DragState,
}

pub struct DragData {
    pub anchor: TileCoord,
    pub transform: DihedralElement,
    pub offset: u8,
    pub external: Option<Course>,
}

impl DragData {
    pub fn isometry_to(&self, pos: TileCoord) -> Isometry {
        Isometry::from_anchors(self.anchor, pos, self.transform)
    }
}

pub enum DragState {
    Selecting(TileCoord),
    Dragging(DragData),
    NoDrag,
}

impl Default for DragState {
    fn default() -> Self {
        Self::NoDrag
    }
}

pub fn selection_rect(
    start: TileCoord,
    pos: TileCoord,
) -> (RangeInclusive<isize>, RangeInclusive<isize>) {
    bounding_rect([start, pos])
}

impl SelectState {
    pub fn clear(&mut self) {
        self.selection.clear();
        self.drag = DragState::NoDrag;
    }

    fn select(&mut self, course: &Course, pos: TileCoord) {
        if course.contains_key(&pos) {
            self.selection.insert(pos);
        }
    }

    pub fn click(&mut self, pos: TileCoord, retain: bool) {
        if !matches!(self.drag, DragState::NoDrag) {
            warn!("Clicked but we're already dragging");
            return;
        }
        if retain || !self.selection.contains(&pos) {
            if !retain {
                self.selection.clear()
            };
            self.drag = DragState::Selecting(pos);
        } else {
            self.drag = DragState::Dragging(DragData {
                anchor: pos,
                transform: DihedralElement::Id,
                offset: 0,
                external: None,
            });
        }
    }

    pub fn finish_rect(&mut self, course: &Course, start: TileCoord, pos: TileCoord) {
        let (xrange, yrange) = selection_rect(start, pos);
        for y in yrange {
            for x in xrange.clone() {
                let pos = TileCoord(x, y);
                self.select(course, pos);
            }
        }
    }

    pub fn toggle_lights(&mut self, course: &mut CourseEdit) {
        match &mut self.drag {
            DragState::Dragging(drag) => drag.offset ^= 1,
            DragState::NoDrag => {
                let mut edit = course.edit();
                for pos in &self.selection {
                    edit.toggle_lights(*pos);
                }
            }
            _ => (),
        }
    }

    pub fn delete(&mut self, course: &mut CourseEdit) {
        if matches!(self.drag, DragState::NoDrag) {
            let mut edit = course.edit();
            for pos in &self.selection {
                edit.remove(*pos);
            }
            self.selection.clear();
        }
    }

    pub fn apply_transform(&mut self, course: &mut CourseEdit, trans: DihedralElement) {
        match &mut self.drag {
            DragState::Dragging(drag) => drag.transform = trans * (drag.transform),
            _ => {
                if self.selection.len() == 1 {
                    let mut edit = course.edit();
                    for pos in &self.selection {
                        edit.apply_transform(*pos, trans);
                    }
                }
            }
        }
    }

    pub fn release(&mut self, course: &mut CourseEdit, pos: TileCoord) {
        let old_drag = std::mem::replace(&mut self.drag, DragState::NoDrag);
        match old_drag {
            DragState::Dragging(drag) => self.apply_drag(course, drag, pos),
            DragState::Selecting(start) => self.finish_rect(course.get_course(), start, pos),
            _ => warn!("Mouse released but we weren't dragging"),
        }
    }

    fn apply_drag(&mut self, course: &mut CourseEdit, drag: DragData, pos: TileCoord) {
        let isom = Isometry::from_anchors(drag.anchor, pos, drag.transform);
        let old_course = course.get_course().clone();
        let mut edit = course.edit();
        let mut new_selection =
            FxHashSet::with_capacity_and_hasher(self.selection.len(), Default::default());
        for &pos in &self.selection {
            if !self.selection.contains(&isom.apply_inverse(pos)) {
                edit.remove(pos);
            }
        }
        for (pos, tile) in drag_tiles(&self.selection, &drag, &old_course, pos) {
            edit.set(pos, tile);
            new_selection.insert(pos);
        }
        self.selection = new_selection;
    }

    pub fn load_external(course: Course) -> Self {
        let center = course_center(&course);
        let pos = TileCoord(center.x.round() as isize, center.y.round() as isize);
        Self {
            selection: Default::default(),
            drag: DragState::Dragging(DragData {
                anchor: pos,
                transform: DihedralElement::Id,
                offset: 0,
                external: Some(course),
            }),
        }
    }
}

type InternalIterator<'a> = <&'a FxHashSet<TileCoord> as IntoIterator>::IntoIter;
type ExternalIterator<'a> = <&'a Course as IntoIterator>::IntoIter;

enum DragIterBase<'a> {
    Internal(InternalIterator<'a>, &'a Course),
    External(ExternalIterator<'a>),
}

impl Iterator for DragIterBase<'_> {
    type Item = (TileCoord, Tile);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Internal(it, course) => it.next().map(|pos| (*pos, *course.get(pos).unwrap())),
            Self::External(it) => it.next().map(|(&pos, &tile)| (pos, tile)),
        }
    }
}

impl<'a> DragIterBase<'a> {
    fn from_drag(
        selection: &'a FxHashSet<TileCoord>,
        drag: &'a DragData,
        course: &'a Course,
    ) -> Self {
        drag.external.as_ref().map_or_else(
            || Self::Internal(selection.iter(), course),
            |c| Self::External(c.iter()),
        )
    }
}

pub fn drag_tiles<'a>(
    selection: &'a FxHashSet<TileCoord>,
    drag: &'a DragData,
    course: &'a Course,
    pos: TileCoord,
) -> impl Iterator<Item = (TileCoord, Tile)> + 'a {
    let isom = drag.isometry_to(pos);
    DragIterBase::from_drag(selection, drag, course).map(move |(from_pos, tile)| {
        (
            isom * from_pos,
            tile.apply_transform(drag.transform, drag.offset),
        )
    })
}
