use std::rc::Rc;

use notan::app::{App, Graphics, Plugins};

use crate::{
    input::Action,
    states::{
        CustomSpecState, DialogResponse, EditState, RaceState, SelectState, SelectStatus,
        SettingsState,
    },
};

use super::{
    credits::credits_screen,
    edit::draw_edit,
    home::draw_home_screen,
    loader::Resources,
    menu::{custom_spec_menu, settings_menu},
    race::draw_race,
    settings::Settings,
};

pub trait Screen {
    fn run(
        self: Box<Self>,
        app: &mut App,
        gfx: &mut Graphics,
        plugins: &mut Plugins,
        res: &Resources,
        settings: &mut Settings,
    ) -> Box<dyn Screen>;
}

impl Screen for EditState {
    fn run(
        mut self: Box<Self>,
        app: &mut App,
        gfx: &mut Graphics,
        plugins: &mut Plugins,
        res: &Resources,
        settings: &mut Settings,
    ) -> Box<dyn Screen> {
        if let Some(command) = draw_edit(app, gfx, plugins, res, settings, &mut self) {
            match command {
                Action::Home => Box::new(SelectState::new(&res.levels)),
                Action::StepForward | Action::Play | Action::FastForward => {
                    let mut new_state =
                        RaceState::new(self.level_data, self.course, self.view_center);
                    new_state.process_command(command, app.timer.elapsed());
                    Box::new(new_state)
                }
                Action::Settings => Box::new(SettingsState::new(self)),
                _ => self,
            }
        } else {
            self
        }
    }
}

impl Screen for SettingsState {
    fn run(
        mut self: Box<Self>,
        app: &mut App,
        gfx: &mut Graphics,
        plugins: &mut Plugins,
        res: &Resources,
        settings: &mut Settings,
    ) -> Box<dyn Screen> {
        let finished = settings_menu(app, gfx, plugins, res, settings, &mut self);
        if finished {
            self.last
        } else {
            self
        }
    }
}

impl Screen for SelectState {
    fn run(
        self: Box<Self>,
        app: &mut App,
        gfx: &mut Graphics,
        plugins: &mut Plugins,
        res: &Resources,
        _settings: &mut Settings,
    ) -> Box<dyn Screen> {
        match draw_home_screen(app, gfx, plugins, res, &self) {
            SelectStatus::Level(n) => Box::new(EditState::new(res.levels[n].clone())),
            SelectStatus::Custom => Box::new(CustomSpecState::new()),
            SelectStatus::Settings => Box::new(SettingsState::new(self)),
            SelectStatus::Credits => Box::new(CreditsState),
            SelectStatus::Idle => self,
        }
    }
}

impl Screen for RaceState {
    fn run(
        mut self: Box<Self>,
        app: &mut App,
        gfx: &mut Graphics,
        plugins: &mut Plugins,
        res: &Resources,
        settings: &mut Settings,
    ) -> Box<dyn Screen> {
        match draw_race(app, gfx, plugins, res, settings, &mut self) {
            Some(Action::Home) => Box::new(SelectState::new(&res.levels)),
            Some(Action::Edit) => Box::new(EditState::new_with_course_edit_and_center(
                self.level_data,
                self.edit,
                self.view_center,
            )),
            Some(Action::Settings) => Box::new(SettingsState::new(self)),
            _ => self,
        }
    }
}

impl Screen for CustomSpecState {
    fn run(
        mut self: Box<Self>,
        _app: &mut App,
        gfx: &mut Graphics,
        plugins: &mut Plugins,
        res: &Resources,
        _settings: &mut Settings,
    ) -> Box<dyn Screen> {
        match custom_spec_menu(gfx, plugins, &mut self) {
            DialogResponse::Accepted(data) => Box::new(EditState::new(Rc::new(data))),
            DialogResponse::Rejected => Box::new(SelectState::new(&res.levels)),
            DialogResponse::Waiting => self,
        }
    }
}

pub struct CreditsState;

impl Screen for CreditsState {
    fn run(
        self: Box<Self>,
        _app: &mut App,
        gfx: &mut Graphics,
        plugins: &mut Plugins,
        res: &Resources,
        _settings: &mut Settings,
    ) -> Box<dyn Screen> {
        if credits_screen(gfx, plugins) {
            Box::new(SelectState::new(&res.levels))
        } else {
            self
        }
    }
}
