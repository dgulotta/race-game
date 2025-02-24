use super::gui::central_panel;
use super::loader::Resources;
use crate::states::{SelectState, SelectStatus};
use notan::app::{App, Color, Graphics, Plugins};
use notan::egui::{self, EguiPluginSugar};

pub fn draw_home_screen(
    _app: &mut App,
    gfx: &mut Graphics,
    plugins: &mut Plugins,
    res: &Resources,
    state: &SelectState,
) -> SelectStatus {
    let mut selection = SelectStatus::Idle;
    let mut output = plugins.egui(|ctx| {
        central_panel(ctx, egui::Align::Min, |ui| {
            ui.heading("Select a level");
            egui::Grid::new("Select grid").show(ui, |ui| {
                ui.heading("Level");
                ui.heading("Area");
                ui.heading("Rounds");
                ui.end_row();
                for (n, lev) in res.levels.iter().enumerate() {
                    let check = if state.solved[n].is_some() {
                        " \u{2714}"
                    } else {
                        ""
                    };
                    let display = format!("{}{check}", &lev.name);
                    if ui.button(&display).clicked() {
                        selection = SelectStatus::Level(n);
                    }
                    if let Some(solve) = state.solved[n] {
                        ui.label(solve.tiles.to_string());
                        ui.label(solve.turns.to_string());
                    }
                    ui.end_row();
                }
            });
            if ui.button("Custom level").clicked() {
                selection = SelectStatus::Custom;
            }
            ui.separator();
            if ui.button("\u{2699} Settings").clicked() {
                selection = SelectStatus::Settings;
            }
            ui.separator();
            if ui.button("Credits").clicked() {
                selection = SelectStatus::Credits;
            }
        });
    });
    output.clear_color(Color::BLACK);
    gfx.render(&output);
    selection
}
