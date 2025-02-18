use regex::Regex;
use std::{rc::Rc, sync::LazyLock, time::Duration};

use notan::math::Vec2;

use crate::{
    course::{course_center, Course, CourseEdit, TileCoord},
    direction::{DihedralElement, Direction},
    input::Action,
    level::{LevelData, SolveData},
    playback::Playback,
    save::{load_course, load_solve, save_course, save_solve},
    simulator::{CarData, Simulator},
    tile::{Tile, TileType},
    tooltip::TooltipState,
    tracker::Tracker,
    ui::{
        export::{make_exporter, FileExport},
        screen::Screen,
    },
};

pub enum TrackSelection {
    Draw(Tile),
    Erase,
    Modify(super::selection::SelectState),
}

impl TrackSelection {
    pub const fn tile_type(&self) -> Option<TileType> {
        match self {
            Self::Draw(t) => Some(t.tile_type),
            _ => None,
        }
    }

    pub fn select(&mut self, tile: TileType) {
        if self.tile_type() != Some(tile) {
            *self = Self::Draw(Tile {
                tile_type: tile,
                transform: DihedralElement::Id,
                offset: 0,
            });
        }
    }

    pub fn reset_selection(&mut self) {
        if let Self::Modify(sel) = self {
            sel.clear();
        }
    }

    pub fn is_action_selected(&self, action: &Action) -> bool {
        match self {
            Self::Draw(t) => action == &Action::SelectTile(t.tile_type),
            Self::Erase => matches!(action, Action::SelectErase),
            Self::Modify(_) => matches!(action, Action::SelectModify),
        }
    }
}

impl Default for TrackSelection {
    fn default() -> Self {
        Self::Modify(Default::default())
    }
}

pub struct SelectState {
    pub solved: Vec<Option<SolveData>>,
}

pub enum SelectStatus {
    Level(usize),
    Custom,
    Settings,
    Credits,
    Idle,
}

impl SelectState {
    pub fn new(data: &[Rc<LevelData>]) -> Self {
        Self {
            solved: data.iter().map(|lvl| load_solve(lvl)).collect(),
        }
    }
}

fn adjust_view_center(center: &mut Vec2, dir: Direction) {
    *center += Vec2::new(dir.dx() as f32, dir.dy() as f32);
}

pub struct EditState {
    pub level_data: Rc<LevelData>,
    pub course: CourseEdit,
    pub track_selection: TrackSelection,
    pub tooltip: Option<TooltipState>,
    pub view_center: Vec2,
    pub show_keys: bool,
    pub copy_dialog_data: Option<Vec<bool>>,
}

impl EditState {
    pub fn save_course(&self) {
        save_course(&self.level_data, self.course.get_course());
    }

    pub fn new_with_course_edit_and_center(
        data: Rc<LevelData>,
        course: CourseEdit,
        view_center: Vec2,
    ) -> Self {
        Self {
            level_data: data.clone(),
            course,
            track_selection: Default::default(),
            tooltip: None,
            view_center,
            show_keys: false,
            copy_dialog_data: None,
        }
    }

    pub fn new_with_course_and_center(
        data: Rc<LevelData>,
        course: Course,
        view_center: Vec2,
    ) -> Self {
        Self::new_with_course_edit_and_center(
            data.clone(),
            CourseEdit::new(course, data),
            view_center,
        )
    }

    pub fn new_with_course(data: Rc<LevelData>, course: Course) -> Self {
        let view_center = course_center(&course);
        Self::new_with_course_and_center(data, course, view_center)
    }

    pub fn new(data: Rc<LevelData>) -> Self {
        let course = load_course(&data).unwrap_or_default();
        Self::new_with_course(data, course)
    }

    pub fn set_tile(&mut self, pos: TileCoord, tile: Tile) {
        self.course.edit().set(pos, tile);
    }

    pub fn remove_tile(&mut self, pos: TileCoord) {
        self.course.edit().remove(pos);
    }

    pub fn undo(&mut self) {
        self.course.undo();
        self.track_selection.reset_selection();
    }

    pub fn redo(&mut self) {
        self.course.redo();
        self.track_selection.reset_selection();
    }

    pub fn process_action(&mut self, action: Action) {
        match action {
            Action::RotCW
            | Action::RotCCW
            | Action::Flip
            | Action::ToggleLights
            | Action::Delete => self.process_transform(action),
            Action::SelectModify => self.track_selection = Default::default(),
            Action::SelectErase => self.track_selection = TrackSelection::Erase,
            Action::SelectTile(t) => self.track_selection.select(t),
            Action::Scroll(dir) => adjust_view_center(&mut self.view_center, dir),
            Action::Undo => self.undo(),
            Action::Redo => self.redo(),
            _ => (),
        }
    }

    pub fn process_transform(&mut self, action: Action) {
        match &mut self.track_selection {
            TrackSelection::Draw(tile) => match action {
                Action::RotCW => tile.transform = DihedralElement::Rot90 * tile.transform,
                Action::RotCCW => tile.transform = DihedralElement::Rot270 * tile.transform,
                Action::Flip => tile.transform = DihedralElement::Flip0 * tile.transform,
                Action::ToggleLights => tile.offset ^= 1,
                Action::Delete => (),
                _ => unreachable!(),
            },
            TrackSelection::Modify(selection) => match action {
                Action::RotCW => {
                    selection.apply_transform(&mut self.course, DihedralElement::Rot90);
                }
                Action::RotCCW => {
                    selection.apply_transform(&mut self.course, DihedralElement::Rot270);
                }
                Action::Flip => {
                    selection.apply_transform(&mut self.course, DihedralElement::Flip0);
                }
                Action::ToggleLights => {
                    selection.toggle_lights(&mut self.course);
                }
                Action::Delete => selection.delete(&mut self.course),
                _ => unreachable!(),
            },
            _ => (),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RaceEndStatus {
    Simulating,
    PopupQueued,
    ShowingPopup,
    Finished,
}

pub struct RaceState {
    pub level_data: Rc<LevelData>,
    pub playback: Playback,
    pub sim: Simulator,
    pub tracker: Tracker,
    pub round: usize,
    pub status: RaceEndStatus,
    pub view_center: Vec2,
    pub edit: CourseEdit,
    pub show_keys: bool,
    pub exporter: Box<dyn FileExport>,
}

impl RaceState {
    pub fn new(level_data: Rc<LevelData>, edit: CourseEdit, view_center: Vec2) -> Self {
        let cars = level_data.cars;
        Self {
            level_data,
            playback: Playback::Paused,
            sim: Simulator::new(edit.get_course().clone(), cars),
            tracker: Tracker::new(cars),
            round: 0,
            status: RaceEndStatus::Simulating,
            view_center,
            edit,
            show_keys: false,
            exporter: make_exporter(),
        }
    }

    pub fn rounds_available(&self) -> usize {
        self.tracker.rounds_available()
    }

    pub fn is_finished(&self) -> bool {
        self.sim.is_finished() || self.tracker.is_loop_detected()
    }

    pub fn sim_round(&mut self) {
        if self.is_finished() {
            return;
        }
        self.sim.run_round();
        for ev in self.sim.events() {
            self.tracker.process_event(ev);
        }
    }

    pub fn forward(&mut self) {
        if self.round >= self.rounds_available() - 1 {
            self.sim_round();
        }
        if self.round < self.rounds_available() - 1 {
            self.round += 1;
        }
    }

    pub fn check_playback_end(&mut self) {
        if self.round == self.rounds_available() - 1 && self.is_finished() {
            self.playback = Playback::Paused;
        }
    }

    pub fn process_command(&mut self, command: Action, time: Duration) {
        match command {
            Action::Seek(n) => self.round = n,
            Action::Start => {
                self.round = 0;
                self.playback = Playback::Paused;
            }
            Action::StepBack => {
                if self.round > 0 {
                    self.round -= 1;
                }
                self.playback = Playback::Paused;
            }
            Action::Pause => self.playback = Playback::Paused,
            Action::Play => self.playback = Playback::Playing(time),
            Action::StepForward => {
                self.forward();
                self.playback = Playback::Paused;
            }
            Action::FastForward => self.playback = Playback::Fast(time),
            Action::End => {
                self.round = self.rounds_available() - 1;
                if self.sim.is_finished() {
                    self.playback = Playback::Paused;
                }
            }
            Action::Keys => self.show_keys = !self.show_keys,
            _ => (),
        }
    }

    pub fn check_advance(&mut self, time: Duration) {
        match self.playback {
            Playback::Playing(t) => {
                if (time - t).as_millis() >= 500 {
                    self.forward();
                    self.playback = Playback::Playing(time);
                }
            }
            Playback::Fast(t) => {
                if (time - t).as_millis() >= 100 {
                    self.forward();
                    self.playback = Playback::Fast(time);
                }
            }
            _ => (),
        }
        self.check_playback_end();
    }

    pub fn get_cars(&self) -> &Vec<CarData> {
        &self.tracker.get_cars()[self.round]
    }

    pub fn get_course(&self) -> &Course {
        self.sim.get_course()
    }

    pub fn solve_data(&self) -> SolveData {
        SolveData {
            tiles: self.sim.get_course().size(),
            turns: self.rounds_available() - 1,
        }
    }

    pub fn check_finished(&mut self) {
        if self.is_finished() {
            self.tracker.compute_final_crashes(self.level_data.cars);
            if &self.level_data.finish == self.tracker.get_finishes() {
                save_solve(&self.level_data, &self.solve_data());
                self.status = RaceEndStatus::PopupQueued;
            } else {
                self.status = RaceEndStatus::Finished;
            }
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.is_finished() && self.round == self.rounds_available() - 1
    }

    pub fn process_action(&mut self, action: Action) {
        if let Action::Scroll(dir) = action {
            adjust_view_center(&mut self.view_center, dir)
        }
    }
}

pub struct CustomSpecState {
    pub cars: usize,
    pub finish: String,
    pub finish_is_valid: bool,
}

pub enum DialogResponse<T> {
    Accepted(T),
    Rejected,
    Waiting,
}

static DIGIT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("[0-9]+").unwrap());

impl Default for CustomSpecState {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomSpecState {
    pub fn new() -> Self {
        Self {
            cars: 0,
            finish: "".to_string(),
            finish_is_valid: true,
        }
    }

    fn parse_finish(cars: usize, finish: &str) -> impl Iterator<Item = Option<usize>> + use<'_> {
        let mut seen = Vec::new();
        seen.resize(cars, false);
        DIGIT_RE
            .find_iter(finish)
            .map(move |s| match s.as_str().parse::<usize>() {
                Ok(n) => {
                    if seen.get(n) == Some(&false) {
                        seen[n] = true;
                        Some(n)
                    } else {
                        None
                    }
                }
                _ => None,
            })
    }

    pub fn check_finish(&mut self) {
        self.finish_is_valid = Self::parse_finish(self.cars, &self.finish).all(|r| r.is_some());
    }

    pub fn get_finish(&self) -> Option<Vec<usize>> {
        Self::parse_finish(self.cars, &self.finish).collect()
    }
}

pub struct SettingsState {
    pub last: Box<dyn Screen>,
    pub action: Option<Action>,
}

impl SettingsState {
    pub fn new(last: Box<dyn Screen>) -> Self {
        Self { last, action: None }
    }
}
