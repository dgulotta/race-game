use std::rc::Rc;

use notan::{
    app::{App, Graphics, Plugins},
    draw::CreateDraw,
    egui::{self, Context, EguiPluginSugar, Rect, Ui},
    math::Vec2,
    prelude::KeyCode,
};
use strum::IntoEnumIterator;

use crate::{
    combine::combine,
    direction::DihedralElement,
    input::Action,
    level::LevelData,
    path::track_tile,
    playback::Playback,
    save::{course_is_nonempty, course_to_vec, load_course},
    selection::{DragState, SelectState, drag_tiles, selection_rect},
    states::{DialogResponse, EditState, TrackSelection},
    tile::TileType,
    tooltip::TooltipState,
};

use super::{
    graphics::{TILE_SIZE, TileGraphics, allocate_ui_space, create_draw_masked, get_draw_offset},
    input::{check_key_press, key_name, mouse_coords, mouse_coords_car},
    loader::{GuiImage, Resources},
    race::{PlaybackPanelState, draw_goal_panel, draw_playback_panel},
    settings::Settings,
};

fn process_keyboard(app: &App, settings: &Settings, state: &mut EditState) -> Option<Action> {
    let mut command = None;
    for &action in settings.keys.keys() {
        if check_key_press(app, settings, action) {
            state.process_action(action);
            if action.can_start_sim() && state.course.get_finish().is_some() {
                command = Some(action);
            }
        }
    }
    command
}

fn process_mouse(
    app: &App,
    settings: &Settings,
    state: &mut EditState,
    offset: &Vec2,
    in_gui: bool,
) {
    let pos = mouse_coords(app, settings, offset);
    if app.mouse.left_was_pressed() {
        state.click_in_gui = in_gui;
    }
    if !state.click_in_gui {
        if app.mouse.left_is_down() && !in_gui {
            match &mut state.track_selection {
                TrackSelection::Erase => {
                    state.remove_tile(pos);
                }
                TrackSelection::Draw(tile) => {
                    state.course.set_single(pos, *tile);
                }
                TrackSelection::Modify(select) => {
                    if app.mouse.left_was_pressed() {
                        let retain = app.keyboard.is_down(KeyCode::ShiftLeft)
                            || app.keyboard.is_down(KeyCode::ShiftRight);
                        select.click(pos, retain);
                    }
                }
                TrackSelection::Path(path) => {
                    if let Some(car_pos) = mouse_coords_car(app, settings, offset) {
                        path.add(car_pos);
                    }
                }
            }
        } else if app.mouse.left_was_released() {
            match &mut state.track_selection {
                TrackSelection::Modify(select) => select.release(&mut state.course, pos),
                TrackSelection::Path(path) => {
                    if !in_gui {
                        let mut edit = state.course.edit();
                        for w in path.path().windows(2) {
                            let (pos, add) = track_tile(w[0], w[1]);
                            let combined = if let Some(orig) = edit.course().get(&pos) {
                                combine(*orig, add, &state.level_data.banned)
                            } else {
                                add
                            };
                            edit.set(pos, combined);
                        }
                    }
                    path.clear();
                }
                _ => (),
            }
        }
    }
    if app.mouse.right_is_down() && !in_gui {
        state.remove_tile(pos);
    }
}

struct TooltipArea {
    pub area: Rect,
    pub selection: TileType,
}

static BUTTON_SIZE: f32 = 51.0;

struct PanelManager<'a> {
    res: &'a Resources,
    settings: &'a Settings,
    state: &'a EditState,
    ui: &'a mut Ui,
    action: Option<Action>,
    tooltip: Option<TooltipArea>,
}

impl PanelManager<'_> {
    fn image_for_action(&self, action: Action) -> &GuiImage {
        match action {
            Action::SelectModify => &self.res.select,
            Action::SelectErase => &self.res.erase.gui_image,
            Action::SelectTile(t) => &self.res.tiles[t].gui_image,
            Action::SelectPath => &self.res.path_icon,
            _ => unreachable!(),
        }
    }

    fn draw_button(&mut self, action: Action) {
        let img = self.image_for_action(action);
        let img_sized =
            egui::Image::new(*img).fit_to_exact_size(egui::Vec2::new(BUTTON_SIZE, BUTTON_SIZE));
        let selected = self.state.track_selection.is_action_selected(&action);
        let response = self
            .ui
            .add(egui::widgets::Button::image(img_sized).selected(selected));
        if response.clicked() && !selected {
            self.action = Some(action);
        }
        let label = action.name_with_key_hint(self.settings);
        match action {
            Action::SelectTile(t) if self.settings.animate_tooltips => {
                response.on_hover_ui(|ui| {
                    let rect = allocate_ui_space(ui, self.settings.zoom.tile_size, 3, 3);
                    ui.label(label);
                    self.tooltip = Some(TooltipArea {
                        area: rect,
                        selection: t,
                    });
                });
            }
            _ => {
                response.on_hover_text(label);
            }
        }
    }
}

fn draw_track_panel(
    res: &Resources,
    settings: &Settings,
    state: &mut EditState,
    ctx: &Context,
) -> (Option<Action>, Option<TooltipArea>) {
    egui::TopBottomPanel::top("Track select")
        .show(ctx, |ui| {
            egui::ScrollArea::horizontal()
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let mut pm = PanelManager {
                            res,
                            settings,
                            state,
                            ui,
                            action: None,
                            tooltip: None,
                        };
                        pm.draw_button(Action::SelectModify);
                        pm.draw_button(Action::SelectErase);
                        pm.draw_button(Action::SelectPath);
                        for t in TileType::iter() {
                            if !state.level_data.banned[t] {
                                pm.draw_button(Action::SelectTile(t));
                            }
                        }
                        (pm.action, pm.tooltip)
                    })
                    .inner
                })
                .inner
        })
        .inner
}

static OVERLAY_ALPHA: f32 = 0.8;

fn draw_course_edit(
    app: &App,
    gfx: &Graphics,
    res: &Resources,
    settings: &Settings,
    state: &EditState,
    mouse_in_gui: bool,
    offset: &Vec2,
) -> notan::draw::Draw {
    let mut graphics = TileGraphics {
        res,
        zoom: settings.zoom.tile_size,
        bg_color: &settings.bg_color,
        draw: gfx.create_draw(),
        round: 1,
    };
    graphics.set_offset(offset);
    if let TrackSelection::Modify(selection) = &state.track_selection {
        graphics.draw_highlights(&selection.selection);
        if let DragState::Selecting(anchor) = &selection.drag {
            let (xrange, yrange) = selection_rect(*anchor, mouse_coords(app, settings, offset));
            graphics.draw_highlights(
                state
                    .course
                    .get_course()
                    .keys()
                    .filter(|pos| xrange.contains(&pos.0) && yrange.contains(&pos.1)),
            );
        }
    }
    graphics.draw_course(state.course.get_course());
    if !mouse_in_gui {
        let pos = mouse_coords(app, settings, offset);
        match &state.track_selection {
            TrackSelection::Draw(tile) => {
                graphics.draw_tile(*tile, pos).alpha(OVERLAY_ALPHA);
            }
            TrackSelection::Erase => {
                graphics
                    .draw_tile_sprite(&res.erase.textures[0], DihedralElement::Id, pos)
                    .alpha(OVERLAY_ALPHA);
            }
            TrackSelection::Modify(selection) => {
                if let DragState::Dragging(drag) = &selection.drag {
                    for (new_pos, tile) in
                        drag_tiles(&selection.selection, drag, state.course.get_course(), pos)
                    {
                        graphics.draw_tile(tile, new_pos).alpha(OVERLAY_ALPHA);
                    }
                }
            }
            TrackSelection::Path(path) => {
                for p in path.path().windows(2) {
                    let (track_pos, tile) = track_tile(p[0], p[1]);
                    graphics.draw_tile(tile, track_pos).alpha(OVERLAY_ALPHA);
                }
                if path.path().is_empty() {
                    if let Some(mpos) = mouse_coords_car(app, settings, offset) {
                        graphics.draw_tile_boundary_arrows(mpos);
                    }
                } else {
                    let p = path.path();
                    if p.len() == 1 {
                        graphics.draw_tile_boundary_arrows(p[0]);
                    } else {
                        graphics.draw_tile_boundary(p[0]);
                        let last = p[p.len() - 1];
                        let track_pos = track_tile(p[p.len() - 2], last).0;
                        graphics.draw_tile_boundary_arrow(last, track_pos);
                    }
                }
            }
        }
    }
    graphics.draw
}

fn tutorial_text(ctx: &Context, text: &str) {
    tutorial_window(ctx, |ui| ui.label(text));
}

fn tutorial_window<R>(ctx: &Context, add_contents: impl FnOnce(&mut Ui) -> R) {
    egui::Window::new("Tutorial")
        .default_width(3.0 * TILE_SIZE)
        .show(ctx, add_contents);
}

pub(crate) fn key_window(ctx: &Context, settings: &Settings, editing: bool) {
    egui::Window::new("Keybindings").show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("keybind").show(ui, |ui| {
                for (k, v) in settings.keys.iter() {
                    let show = if editing {
                        k.is_active_when_editing()
                    } else {
                        k.is_active_when_racing()
                    };
                    if show {
                        ui.label(k.name());
                        ui.label(key_name(*v));
                        ui.end_row()
                    }
                }
            });
        });
    });
}

fn tutorial_new_tiles(state: &EditState, ctx: &Context) {
    if state.track_selection.tile_type().is_none() && state.course.get_course().is_empty() {
        tutorial_text(ctx, "New track types are available.");
    }
}

fn draw_tutorial(res: &Resources, settings: &Settings, state: &EditState, ctx: &Context) {
    match state.level_data.tutorial {
        Some(0) => {
            if state.course.get_finish().is_some() {
                tutorial_text(
                    ctx,
                    "The current goal is displayed in the panel on the left.\n\nThe goal for this level is for none of the cars to finish.\n\nClick the \u{23f5} button below to start the race.",
                );
            } else if state.track_selection.tile_type() == Some(TileType::Finish) {
                tutorial_text(
                    ctx,
                    "Now click in the center area to place the start/finish line.",
                );
            } else {
                tutorial_window(ctx, |ui| {
                    ui.label("Your job is to design race tracks.\n\nEvery track needs a start / finish line.\n\nTo select the start/finish line, click the button above with this icon:");
                    let img = egui::Image::new(res.tiles[TileType::Finish].gui_image)
                        .max_width(TILE_SIZE);
                    ui.add(img);
                });
            }
        }
        Some(1) => {
            if state.track_selection.tile_type().is_some() {
                tutorial_text(
                    ctx,
                    &format!(
                        "Press {} or {} to rotate the track before placing it.  Press {} to flip the track horizontally.  Press {} to reverse the track direction.\n\nClick the \u{1f5ae} button below to see more keybindings, or the \u{2699} button to change the keybindings.",
                        key_name(settings.keys[&Action::RotCCW]),
                        key_name(settings.keys[&Action::RotCW]),
                        key_name(settings.keys[&Action::Flip]),
                        key_name(settings.keys[&Action::Reverse]),
                    ),
                );
            } else if matches!(state.track_selection, TrackSelection::Path(_)) {
                tutorial_text(ctx, "Click and drag to place track pieces.");
            } else {
                tutorial_window(ctx, |ui| {
                    ui.label("The goal for this level is for the cars to finish the race in the same order that they started it.\n\nHover the mouse over the pieces of track in the top panel to see what they do.\n\nTo draw many pieces of track quickly, use this tool:");
                    let img = egui::Image::new(res.path_icon).max_width(TILE_SIZE);
                    ui.add(img);
                });
            }
        }
        Some(2) => {
            if state
                .track_selection
                .tile_type()
                .is_some_and(TileType::has_lights)
            {
                tutorial_text(
                    ctx,
                    &format!(
                        "The selected track has lights.  Press {} to toggle the lights.",
                        key_name(settings.keys[&Action::ToggleLights])
                    ),
                )
            } else {
                tutorial_new_tiles(state, ctx);
            }
        }
        Some(3) => {
            tutorial_new_tiles(state, ctx);
        }
        Some(4) => {
            if matches!(
                state.track_selection.tile_type(),
                Some(TileType::YieldIntersection)
            ) {
                tutorial_text(
                    ctx,
                    &format!(
                        "The selected track does not have lights, but you can press {} to move the yield sign.",
                        key_name(settings.keys[&Action::ToggleLights])
                    ),
                )
            }
        }
        _ => (),
    }
}

fn draw_copy_dialog<'a>(
    ctx: &Context,
    available: &[bool],
    level_data: &'a [Rc<LevelData>],
) -> DialogResponse<&'a LevelData> {
    let mut selected = DialogResponse::Waiting;
    egui::Window::new("Copy").show(ctx, |ui| {
        ui.label("Copy a level:");
        for (&saved, lev) in available.iter().zip(level_data) {
            if saved && ui.button(&lev.name).clicked() {
                selected = DialogResponse::Accepted(lev.as_ref());
            }
        }
        if ui.button("Close").clicked() {
            selected = DialogResponse::Rejected;
        }
    });
    selected
}

fn process_debug_commands(app: &App, state: &mut EditState) {
    if app.keyboard.was_pressed(KeyCode::F1) {
        let mut lev = std::collections::BTreeMap::new();
        let course = course_to_vec(state.course.get_course());
        lev.insert(state.level_data.name.clone(), course);
        println!("{}", toml::to_string(&lev).unwrap());
    } else if app.keyboard.was_pressed(KeyCode::F2) {
        let solutions =
            crate::save::courses_from_toml(include_str!("../../res/solutions.toml")).unwrap();
        if let Some(course) = solutions.get(&state.level_data.name) {
            state.course.set_course(course.clone());
        }
    }
}

pub fn draw_edit(
    app: &mut App,
    gfx: &mut Graphics,
    plugins: &mut Plugins,
    res: &Resources,
    settings: &Settings,
    state: &mut EditState,
) -> Option<Action> {
    let mut play_command = process_keyboard(app, settings, state);
    if cfg!(feature = "debug") {
        process_debug_commands(app, state);
    }
    let mut mouse_in_gui = false;
    let mut tooltip: Option<TooltipArea> = None;
    let mut draw_rect = Rect::NOTHING;
    let output = plugins.egui(|ctx| {
        draw_goal_panel(&state.level_data, None, None, ctx);
        play_command = draw_playback_panel(
            PlaybackPanelState::Editing(state.course.get_finish().is_some()),
            settings,
            ctx,
        )
        .or(play_command);
        if matches!(play_command, Some(Action::Copy)) {
            let saved: Vec<bool> = res
                .levels
                .iter()
                .map(|lvl| course_is_nonempty(lvl.as_ref()))
                .collect();
            state.copy_dialog_data = Some(saved);
        }
        if matches!(play_command, Some(Action::Keys)) {
            state.show_keys = !state.show_keys;
        }
        if settings.tutorial {
            draw_tutorial(res, settings, state, ctx);
        }
        if state.show_keys {
            key_window(ctx, settings, true);
        }
        if let Some(cdd) = &state.copy_dialog_data {
            match draw_copy_dialog(ctx, cdd, &res.levels) {
                DialogResponse::Accepted(lev) => {
                    if let Some(course) = load_course(lev) {
                        if !course.is_empty() {
                            state.track_selection =
                                TrackSelection::Modify(SelectState::load_external(course));
                            state.copy_dialog_data = None;
                        }
                    }
                }
                DialogResponse::Rejected => state.copy_dialog_data = None,
                _ => (),
            }
        }
        draw_rect = ctx.available_rect() * ctx.zoom_factor();
        let (action, new_tooltip) = draw_track_panel(res, settings, state, ctx);
        tooltip = new_tooltip;
        if let Some(act) = action {
            state.process_action(act);
        }
        mouse_in_gui = ctx.is_pointer_over_area();
    });
    let offset = get_draw_offset(&state.view_center, &draw_rect);
    process_mouse(app, settings, state, &offset, mouse_in_gui);
    let draw = draw_course_edit(app, gfx, res, settings, state, mouse_in_gui, &offset);

    gfx.render(&draw);
    gfx.render(&output);

    if let Some(tool_area) = tooltip {
        if state.tooltip.as_ref().map(|t| t.tile) != Some(tool_area.selection) {
            state.tooltip = Some(TooltipState::new(tool_area.selection));
        }
        let tool_state = state.tooltip.as_mut().unwrap();
        let time = app.timer.elapsed();
        let round = if settings.smooth_animation
            && !tool_state.animations.is_empty()
            && time < tool_state.last_sim_time + Playback::Playing.frame_duration()
        {
            tool_state.sim.get_round() - 1
        } else {
            tool_state.sim.get_round()
        };
        let mut graphics = TileGraphics {
            res,
            zoom: settings.zoom.tile_size,
            bg_color: &settings.bg_color,
            draw: create_draw_masked(gfx, &tool_area.area),
            round,
        };
        for (pos, tile) in tool_state.sim.get_course().iter() {
            if (0..=2).contains(&pos.0) && (0..=2).contains(&pos.1) {
                graphics.draw_tile(*tile, *pos);
            }
        }
        if time - tool_state.last_sim_time >= Playback::Playing.frame_duration() {
            tool_state.advance(time);
        }
        if settings.smooth_animation && !tool_state.animations.is_empty() {
            let t = (time - tool_state.last_sim_time)
                .div_duration_f32(Playback::Playing.frame_duration());
            for anim in tool_state.animations.iter() {
                graphics.draw_car_smooth(anim.id, &anim.position_at_time(t));
            }
        } else {
            for car in &tool_state.cars {
                graphics.draw_car(car);
            }
        }
        gfx.render(&graphics.draw);
    }
    play_command
}
