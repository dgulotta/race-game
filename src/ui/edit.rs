use std::rc::Rc;

use notan::{
    app::{App, Graphics, Plugins},
    draw::{CreateDraw, DrawShapes},
    egui::{self, Context, EguiPluginSugar, Rect, Sense, Ui},
    math::{Mat3, Vec2},
    prelude::KeyCode,
};
use strum::IntoEnumIterator;

use crate::{
    direction::DihedralElement,
    input::Action,
    level::LevelData,
    save::{course_is_nonempty, load_course},
    selection::{drag_tiles, selection_rect, DragState, SelectState},
    states::{DialogResponse, EditState, TrackSelection},
    tile::TileType,
    tooltip::TooltipState,
};

use super::{
    graphics::{
        draw_car, draw_course, draw_highlights, draw_tile, draw_tile_sprite, get_draw_offset,
        set_offset, TILE_SIZE,
    },
    input::{check_key_press, key_name, mouse_coords},
    loader::{GuiImage, Resources},
    race::{draw_goal_panel, draw_playback_panel, PlaybackPanelState},
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

fn process_mouse(app: &App, state: &mut EditState, offset: &Vec2, in_gui: bool) {
    let pos = mouse_coords(app, offset);
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
                        let retain = app.keyboard.is_down(KeyCode::LShift)
                            || app.keyboard.is_down(KeyCode::RShift);
                        select.click(pos, retain);
                    }
                }
            }
        } else if app.mouse.left_was_released() {
            if let TrackSelection::Modify(select) = &mut state.track_selection {
                select.release(&mut state.course, pos);
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

static BUTTON_SIZE: f32 = 65.0;

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
            Action::SelectModify => &self.res.gui_tiles[0],
            Action::SelectErase => &self.res.gui_tiles[1],
            Action::SelectTile(t) => &self.res.gui_tiles[(t as usize) + 2],
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
                    let tool_resp = ui.allocate_response(
                        egui::Vec2::new(3.0 * TILE_SIZE, 3.0 * TILE_SIZE) / ui.ctx().zoom_factor(),
                        Sense::hover(),
                    );
                    ui.label(label);
                    self.tooltip = Some(TooltipArea {
                        area: tool_resp.rect * ui.ctx().zoom_factor(),
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
                        for t in TileType::iter() {
                            pm.draw_button(Action::SelectTile(t));
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
    state: &EditState,
    mouse_in_gui: bool,
    offset: &Vec2,
) -> notan::draw::Draw {
    let mut draw = gfx.create_draw();
    set_offset(&mut draw, offset);
    if let TrackSelection::Modify(selection) = &state.track_selection {
        draw_highlights(&mut draw, &selection.selection);
        if let DragState::Selecting(anchor) = &selection.drag {
            let (xrange, yrange) = selection_rect(*anchor, mouse_coords(app, offset));
            draw_highlights(
                &mut draw,
                state
                    .course
                    .get_course()
                    .keys()
                    .filter(|pos| xrange.contains(&pos.0) && yrange.contains(&pos.1)),
            );
        }
    }
    draw_course(&mut draw, res, state.course.get_course(), 1);
    if !mouse_in_gui {
        let pos = mouse_coords(app, offset);
        match &state.track_selection {
            TrackSelection::Draw(tile) => {
                draw_tile(&mut draw, *tile, res, pos, 1).alpha(OVERLAY_ALPHA);
            }
            TrackSelection::Erase => {
                draw_tile_sprite(&mut draw, &res.erase, DihedralElement::Id, pos)
                    .alpha(OVERLAY_ALPHA);
            }
            TrackSelection::Modify(selection) => {
                if let DragState::Dragging(drag) = &selection.drag {
                    for (new_pos, tile) in
                        drag_tiles(&selection.selection, drag, state.course.get_course(), pos)
                    {
                        draw_tile(&mut draw, tile, res, new_pos, 1).alpha(OVERLAY_ALPHA);
                    }
                }
            }
        }
    }
    draw
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

fn draw_tutorial(res: &Resources, settings: &Settings, state: &EditState, ctx: &Context) {
    match state.level_data.tutorial {
        Some(0) => {
            if state.course.get_finish().is_some() {
                tutorial_text(ctx,"The current goal is displayed in the panel on the left.\n\nThe goal for this level is for none of the cars to finish.\n\nClick the \u{23f5} button below to start the race.");
            } else if state.track_selection.tile_type() == Some(TileType::Finish) {
                tutorial_text(
                    ctx,
                    "Now click in the center area to place the start/finish line.",
                );
            } else {
                tutorial_window(ctx, |ui| {
                    ui.label("Your job is to design race tracks.\n\nEvery track needs a start / finish line.\n\nTo select the start/finish line, click the button above with this icon:");
                    let img = egui::Image::new(res.gui_tiles[4]).max_width(TILE_SIZE);
                    ui.add(img);
                });
            }
        }
        Some(1) => {
            if state.track_selection.tile_type().is_some() {
                tutorial_text(
                    ctx,
                    &format!("Press {} or {} to rotate the track before placing it.  Press {} to flip the track.\n\nClick the \u{1f5ae} button below to see more keybindings, or the \u{2699} button to change the keybindings.",
                    key_name(settings.keys[&Action::RotCCW]),
                    key_name(settings.keys[&Action::RotCW]),
                    key_name(settings.keys[&Action::Flip])),
                );
            } else {
                tutorial_text(ctx,"The goal for this level is for the cars to finish the race in the same order that they started it.");
            }
        }
        Some(2) => {
            if state
                .track_selection
                .tile_type()
                .map_or(false, TileType::has_lights)
            {
                tutorial_text(
                    ctx,
                    &format!(
                        "The selected track has lights.  Press {} to toggle the lights.",
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

pub fn draw_edit(
    app: &mut App,
    gfx: &mut Graphics,
    plugins: &mut Plugins,
    res: &Resources,
    settings: &Settings,
    state: &mut EditState,
) -> Option<Action> {
    let mut play_command = process_keyboard(app, settings, state);
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
        draw_rect = ctx.available_rect();
        let (action, new_tooltip) = draw_track_panel(res, settings, state, ctx);
        tooltip = new_tooltip;
        if let Some(act) = action {
            state.process_action(act);
        }
        mouse_in_gui = ctx.is_pointer_over_area();
    });
    let offset = get_draw_offset(&state.view_center, &draw_rect);
    process_mouse(app, state, &offset, mouse_in_gui);
    let draw = draw_course_edit(app, gfx, res, state, mouse_in_gui, &offset);

    gfx.render(&draw);
    gfx.render(&output);

    if let Some(tool_area) = tooltip {
        let mut mask = gfx.create_draw();
        mask.rect(
            (tool_area.area.min.x, tool_area.area.min.y),
            (tool_area.area.width(), tool_area.area.height()),
        );
        let mut tool_draw = gfx.create_draw();
        if state.tooltip.as_ref().map(|t| t.tile) != Some(tool_area.selection) {
            state.tooltip = Some(TooltipState::new(tool_area.selection));
        }
        let tool_state = state.tooltip.as_mut().unwrap();
        tool_draw.transform().push(Mat3::from_translation(Vec2::new(
            tool_area.area.min.x,
            tool_area.area.min.y,
        )));
        tool_draw.mask(Some(&mask));
        tool_draw.rect(
            (0.0, 0.0),
            (tool_area.area.width(), tool_area.area.height()),
        );
        for (pos, tile) in tool_state.sim.get_course().iter() {
            if (0..=2).contains(&pos.0) && (0..=2).contains(&pos.1) {
                draw_tile(&mut tool_draw, *tile, res, *pos, tool_state.sim.get_round());
            }
        }
        let time = app.timer.elapsed();
        if (time - tool_state.last_sim_time).as_millis() >= 500 {
            tool_state.advance(time);
        }
        for car in &tool_state.cars {
            draw_car(&mut tool_draw, res, car);
        }
        gfx.render(&tool_draw);
    }
    play_command
}
