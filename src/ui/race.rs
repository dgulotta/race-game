use std::io::Cursor;

use crate::{
    course::bounding_rect,
    direction::Direction,
    input::Action,
    level::LevelData,
    playback::Playback,
    states::{RaceEndStatus, RaceState},
    tracker::compute_not_finishing,
};
use notan::{
    app::{App, Graphics, Plugins},
    draw::CreateDraw,
    egui::{self, Button, Context, EguiPluginSugar, Rect, Slider, Ui},
    math::{Affine2, Mat2, Vec2},
};
use strum::IntoEnumIterator;

use super::{
    edit::key_window,
    graphics::{draw_car, draw_car_number, draw_course, get_draw_offset, set_offset, TILE_SIZE},
    input::check_key_press,
    loader::Resources,
    settings::Settings,
};

#[derive(Copy, Clone)]
pub enum PlaybackPanelState {
    Editing(bool),
    Viewing(Playback, usize, usize),
}

impl PlaybackPanelState {
    fn editing(&self) -> bool {
        matches!(self, Self::Editing(_))
    }
    fn viewing(&self) -> bool {
        !self.editing()
    }
    fn play_pressed(&self) -> bool {
        matches!(self, Self::Viewing(Playback::Playing(_), _, _))
    }
    fn pause_pressed(&self) -> bool {
        matches!(self, Self::Viewing(Playback::Paused, _, _))
    }
    fn pause_enabled(&self) -> bool {
        match self {
            Self::Viewing(p, _, _) => *p != Playback::Paused,
            _ => false,
        }
    }
    fn play_enabled(&self) -> bool {
        match self {
            Self::Editing(ready) => *ready,
            Self::Viewing(_, r, m) => *r < m - 1,
        }
    }
    fn back_enabled(&self) -> bool {
        match self {
            Self::Viewing(_, r, _) => *r > 0,
            _ => false,
        }
    }
}

struct PlaybackPanelData<'a, 'b> {
    ui: &'a mut Ui,
    settings: &'b Settings,
    command: Option<Action>,
}

impl PlaybackPanelData<'_, '_> {
    fn add_button(&mut self, action: Action, text: &str, enabled: bool) {
        self.add_button_selected(action, text, enabled, false);
    }

    fn add_button_selected(&mut self, action: Action, text: &str, enabled: bool, selected: bool) {
        if self
            .ui
            .add_enabled(enabled, Button::new(text).selected(selected))
            .on_hover_text(action.name_with_key_hint(self.settings))
            .clicked()
        {
            self.command = Some(action);
        }
    }
}

pub fn draw_playback_panel(
    state: PlaybackPanelState,
    settings: &Settings,
    ctx: &Context,
) -> Option<Action> {
    let play_enabled = state.play_enabled();
    let (round_old, round_max) = match state {
        PlaybackPanelState::Viewing(_, r, m) => (r, m - 1),
        _ => (0, 1),
    };
    egui::TopBottomPanel::bottom("Playback")
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let mut pd = PlaybackPanelData {
                    ui,
                    settings,
                    command: None,
                };
                pd.add_button(Action::Home, "\u{1f3e0}", true);
                pd.add_button(Action::Settings, "\u{2699}", true);
                pd.add_button(Action::Keys, "\u{1f5ae}", true);
                pd.add_button(Action::Copy, "\u{1f4cb}", !state.viewing());
                pd.add_button(Action::Edit, "\u{270f}", state.viewing());
                pd.add_button(Action::Start, "\u{23ee}", state.back_enabled());
                pd.add_button(Action::StepBack, "|\u{23f4}", state.back_enabled());
                pd.add_button_selected(
                    Action::Pause,
                    "\u{23f8}",
                    state.pause_enabled(),
                    state.pause_pressed(),
                );
                pd.add_button(Action::StepForward, "\u{23f5}|", play_enabled);
                pd.add_button_selected(
                    Action::Play,
                    "\u{23f5}",
                    play_enabled,
                    state.play_pressed(),
                );
                pd.add_button(Action::FastForward, "\u{23e9}|", play_enabled);
                pd.add_button(Action::End, "\u{23ed}", play_enabled && state.viewing());
                let mut round_new = round_old;
                pd.ui
                    .add_enabled(state.viewing(), Slider::new(&mut round_new, 0..=round_max));
                if round_new != round_old {
                    pd.command = Some(Action::Seek(round_new));
                }
                pd.command
            })
            .inner
        })
        .inner
}

pub fn draw_goal_panel(
    level: &LevelData,
    finished: Option<&Vec<usize>>,
    crashed: Option<&[bool]>,
    ctx: &Context,
) {
    egui::SidePanel::left("Goal").show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Finish order");
            ui.columns(2, |col| {
                col[0].label("Goal");
                for n in &level.finish {
                    col[0].label(n.to_string());
                }
                col[1].label("Actual");
                if let Some(fin) = finished {
                    for n in fin {
                        col[1].label(n.to_string());
                    }
                }
            });
            ui.separator();
            ui.heading("Not finishing");
            ui.columns(2, |col| {
                col[0].label("Goal");
                let not_finishing = compute_not_finishing(level.cars, &level.finish);
                for (n, nf) in not_finishing.iter().enumerate() {
                    if *nf {
                        col[0].label(n.to_string());
                    }
                }
                col[1].label("Actual");
                if let Some(cr) = crashed {
                    for (n, c) in cr.iter().enumerate() {
                        if *c {
                            col[1].label(n.to_string());
                        }
                    }
                }
            })
        });
    });
}

fn gfx_size_for(tiles: isize) -> u32 {
    ((tiles as f32) * TILE_SIZE).round() as u32
}

fn anh(s: String) -> anyhow::Error {
    anyhow::anyhow!("{}", s)
}

fn make_animation(
    gfx: &mut Graphics,
    res: &Resources,
    state: &RaceState,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut out = Cursor::new(Vec::new());
    let course = state.sim.get_course();
    let (xrange, yrange) = bounding_rect(course.keys());
    let width = gfx_size_for(xrange.end() - xrange.start() + 1);
    let height = gfx_size_for(yrange.end() - yrange.start() + 1);
    let texture = gfx
        .create_render_texture(width, height)
        .build()
        .map_err(anh)?;
    let mut pix = vec![0; 4 * (width as usize) * (height as usize)];
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder
        .set_animated(state.tracker.rounds_available() as u32, 0)
        .unwrap();
    let mut writer = encoder.write_header().unwrap();
    writer.set_frame_delay(1, 2).unwrap();
    let xoff = (*xrange.start() as f32) * TILE_SIZE;
    let yoff = ((*yrange.end() + 1) as f32) * TILE_SIZE;
    let aff = Affine2::from_mat2_translation(
        Mat2::from_diagonal(Vec2::new(1.0, -1.0)),
        Vec2::new(-xoff, yoff),
    );
    for round in 0..state.tracker.rounds_available() {
        let mut draw = texture.create_draw();
        draw.transform().push(aff.into());
        draw_course(&mut draw, res, course, round);
        for car in &state.tracker.get_cars()[round] {
            draw_car(&mut draw, res, car);
            draw_car_number(&mut draw, res, car);
        }
        gfx.render_to(&texture, &draw);
        gfx.read_pixels(&texture).read_to(&mut pix).map_err(anh)?;
        writer.write_image_data(&pix).unwrap();
    }
    writer.finish().unwrap();
    Ok(out.into_inner())
}

pub fn show_success(
    gfx: &mut Graphics,
    res: &Resources,
    state: &mut RaceState,
    ctx: &Context,
) -> Option<Action> {
    egui::Window::new("Success!")
        .show(ctx, |ui| {
            let data = state.solve_data();
            let report = format!("Rounds: {}\nTiles used: {}", data.turns, data.tiles);
            let mut command = None;
            ui.label(report);
            ui.horizontal(|ui| {
                if ui.button("\u{1f3e0} Select level").clicked() {
                    command = Some(Action::Home);
                }
                if ui.button("\u{270f} Edit track").clicked() {
                    command = Some(Action::Edit);
                }
                if ui.button("Continue watching").clicked() {
                    state.status = RaceEndStatus::Finished;
                }
                if ui.button("Save replay").clicked() {
                    if let Ok(bytes) = make_animation(gfx, res, state) {
                        let _ = state.exporter.set_save_action(
                            Box::new(move |w| {
                                w.write_all(&bytes)?;
                                Ok(())
                            }),
                            "race.png",
                        );
                    }
                }
            });
            command
        })?
        .inner?
}

static PLAYBACK_ACTIONS: &[Action] = &[
    Action::Start,
    Action::StepBack,
    Action::Pause,
    Action::StepForward,
    Action::Play,
    Action::FastForward,
    Action::End,
];

pub fn draw_race(
    app: &mut App,
    gfx: &mut Graphics,
    plugins: &mut Plugins,
    res: &Resources,
    settings: &Settings,
    state: &mut RaceState,
) -> Option<Action> {
    for dir in Direction::iter() {
        let action = Action::Scroll(dir);
        if check_key_press(app, settings, action) {
            state.process_action(action);
        }
    }
    let time = app.timer.elapsed();
    for &act in PLAYBACK_ACTIONS {
        if check_key_press(app, settings, act) {
            state.process_command(act, time);
        }
    }
    if state.status == RaceEndStatus::Simulating {
        while !state.is_finished() {
            state.sim_round();
        }
        state.check_finished();
    }
    if state.status == RaceEndStatus::PopupQueued && state.is_at_end() {
        state.status = RaceEndStatus::ShowingPopup;
    }
    let mut command: Option<Action> = None;
    let mut draw_rect = Rect::NOTHING;
    let output = plugins.egui(|ctx| {
        draw_goal_panel(
            &state.level_data,
            Some(state.tracker.get_finishes()),
            Some(state.tracker.get_crashes()),
            ctx,
        );
        let pps = PlaybackPanelState::Viewing(
            state.playback,
            state.round,
            state.tracker.rounds_available(),
        );
        command = draw_playback_panel(pps, settings, ctx);
        if state.status == RaceEndStatus::ShowingPopup {
            command = command.or(show_success(gfx, res, state, ctx));
        }
        draw_rect = ctx.available_rect();
        if state.show_keys {
            key_window(ctx, settings, false);
        }
        let _ = state.exporter.update(ctx);
    });
    if let Some(cmd) = command {
        state.process_command(cmd, time);
    }
    state.check_advance(time);
    let mut draw = gfx.create_draw();
    let offset = get_draw_offset(&state.view_center, &draw_rect);
    set_offset(&mut draw, &offset);
    draw_course(&mut draw, res, state.sim.get_course(), state.round);
    for car in state.get_cars() {
        draw_car(&mut draw, res, car);
        draw_car_number(&mut draw, res, car);
    }
    gfx.render(&draw);
    gfx.render(&output);
    command
}
