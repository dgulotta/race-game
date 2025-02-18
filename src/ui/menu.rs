use notan::{
    app::{App, Color, Graphics, Plugins},
    egui::{self, EguiPluginSugar, Grid, RichText},
};

use super::{graphics::TILE_SIZE, input::key_name, settings::Settings};
use crate::{
    input::Action,
    level::LevelData,
    save::save_or_log_err,
    states::{CustomSpecState, DialogResponse},
};

pub fn settings_menu(
    app: &App,
    gfx: &mut Graphics,
    plugins: &mut Plugins,
    settings: &mut Settings,
    action: &mut Option<Action>,
) -> bool {
    let mut exited = false;
    let mut output = plugins.egui(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            match action {
                Some(a) => {
                    ui.heading("Press any key");
                    if let Some(key) = app.keyboard.pressed.iter().next() {
                        settings.keys.insert(*a, *key);
                        *action = None;
                    }
                }
                None => {
                    //egui::Window::new("Settings").show(ctx, |ui| {
                    ui.heading("Keyboard commands");
                    Grid::new("key grid").show(ui, |ui| {
                        //ui.columns(4, |col| {
                        for (n, (k, v)) in settings.keys.iter().enumerate() {
                            ui.label(k.name());
                            if ui.button(key_name(*v)).clicked() {
                                *action = Some(*k);
                            }
                            if n & 1 != 0 {
                                ui.end_row();
                            }
                        }
                    });
                    ui.add_space(20.0);
                    ui.heading("Help");
                    ui.checkbox(&mut settings.tutorial, "Tutorial");
                    ui.checkbox(&mut settings.animate_tooltips, "Animated tooltips");
                    ui.add_space(20.0);
                    exited = ui.button(RichText::new("Close").heading()).clicked();
                }
            }
        });
    });
    output.clear_color(Color::BLACK);
    gfx.render(&output);
    if exited {
        save_or_log_err("settings", settings, "failed to save settings");
    }
    exited
}

pub fn custom_spec_menu(
    gfx: &mut Graphics,
    plugins: &mut Plugins,
    state: &mut CustomSpecState,
) -> DialogResponse<LevelData> {
    let mut status = DialogResponse::Waiting;
    let mut output = plugins.egui(|ctx| {
        egui::Window::new("Level settings").show(ctx, |ui| {
            Grid::new("custom level grid").show(ui, |ui| {
                ui.label("# of cars");
                let mut changed = ui
                    .add(egui::DragValue::new(&mut state.cars).range(0..=100))
                    .changed();
                ui.end_row();
                ui.label("Finish order");
                changed |= ui
                    .add(
                        egui::TextEdit::multiline(&mut state.finish)
                            .min_size(egui::Vec2::new(3.0 * TILE_SIZE, TILE_SIZE)),
                    )
                    .changed();
                if changed {
                    state.check_finish();
                }
            });
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(state.finish_is_valid, egui::Button::new("Ok"))
                    .clicked()
                {
                    let lvl = LevelData {
                        name: "Custom Level".to_string(),
                        cars: state.cars,
                        finish: state.get_finish().unwrap(),
                        tutorial: None,
                    };
                    status = DialogResponse::Accepted(lvl);
                }
                if ui.button("Cancel").clicked() {
                    status = DialogResponse::Rejected;
                }
            })
        });
    });
    output.clear_color(Color::BLACK);
    gfx.render(&output);
    status
}
