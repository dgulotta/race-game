use notan::{
    app::{App, Color, Graphics, Plugins},
    egui::{self, Context, EguiPluginSugar, Grid, Rgba, RichText, Ui},
};

use super::{
    graphics::TILE_SIZE,
    gui::central_panel,
    input::key_name,
    loader::Resources,
    settings::{Settings, ZoomSettings},
};
use crate::{
    input::Action,
    level::LevelData,
    save::save_or_log_err,
    states::{CustomSpecState, DialogResponse, SettingsMenu, SettingsState},
};

pub fn apply_zoom_settings(settings: &Settings, ctx: &Context) {
    ctx.set_zoom_factor(settings.zoom.font_size);
}

const SIZES: &[f32] = &[0.50, 0.75, 1.00, 1.25, 1.50, 1.75, 2.00, 3.00, 4.00];

fn zoom_string(value: f32) -> String {
    format!("{:.0}%", value * 100.0)
}

fn size_combo(ui: &mut Ui, name: &str, value: &mut f32) {
    egui::ComboBox::from_label(name)
        .selected_text(zoom_string(*value))
        .show_ui(ui, |ui| {
            for sz in SIZES {
                ui.selectable_value(value, *sz, zoom_string(*sz));
            }
        });
}

fn display_settings(
    app: &mut App,
    res: &Resources,
    settings: &mut Settings,
    _temp_zoom: &mut ZoomSettings,
    ui: &mut Ui,
) {
    let old_font_size = settings.zoom.font_size;
    ui.checkbox(&mut settings.smooth_animation, "Smooth animation");
    ui.horizontal(|ui| {
        ui.label("Track background");
        ui.color_edit_button_rgb(&mut settings.bg_color);
    });
    ui.horizontal(|ui| {
        ui.label("UI theme:");
        settings.ui_theme.radio_buttons(ui);
    });
    if cfg!(not(target_arch = "wasm32")) {
        let mut full = app.window().is_fullscreen();
        ui.checkbox(&mut full, "Fullscreen");
        app.window().set_fullscreen(full);
    }
    size_combo(ui, "Road tile size", &mut settings.zoom.tile_size);
    size_combo(ui, "UI size", &mut settings.zoom.font_size);
    apply_zoom_settings(settings, ui.ctx());
    /*
    ui.add(
        egui::Slider::new(&mut temp_zoom.tile_size, 0.5..=4.0)
            .text("Road tile size")
            .logarithmic(true),
    );
    ui.add(
        egui::Slider::new(&mut temp_zoom.font_size, 0.5..=4.0)
            .text("UI size")
            .logarithmic(true),
    );
    if ui.button("Apply size settings").clicked() {
        settings.zoom = temp_zoom.clone();
        apply_zoom_settings(settings, ui.ctx());
    }
    */
    let tile_size = settings.tile_size() / old_font_size;
    let img = egui::Image::new(res.sample)
        .fit_to_exact_size(egui::Vec2::new(3.0 * tile_size, 2.0 * tile_size))
        .bg_fill(Rgba::from_rgb(
            settings.bg_color[0],
            settings.bg_color[1],
            settings.bg_color[2],
        ));
    ui.add(img);
}

pub fn settings_menu(
    app: &mut App,
    gfx: &mut Graphics,
    plugins: &mut Plugins,
    res: &Resources,
    settings: &mut Settings,
    state: &mut SettingsState,
) -> bool {
    let mut exit = false;
    let mut back = false;
    let mut output = plugins.egui(|ctx| match &mut state.menu {
        SettingsMenu::Main => {
            exit = settings_menu_main(settings, state, ctx);
        }
        SettingsMenu::Keys => settings_menu_keys(settings, state, ctx),
        SettingsMenu::ChooseKey(a) => {
            back = settings_menu_choose_key(app, settings, ctx, a);
        }
        SettingsMenu::Display(window) => {
            back = settings_menu_display(app, res, settings, window, ctx);
        }
        SettingsMenu::Help => settings_menu_help(settings, state, ctx),
    });
    if back {
        state.menu = SettingsMenu::Main;
    }
    output.clear_color(Color::BLACK);
    gfx.render(&output);
    if exit {
        save_or_log_err("settings", settings, "failed to save settings");
    }
    exit
}

pub fn settings_menu_main(
    settings: &mut Settings,
    state: &mut SettingsState,
    ctx: &Context,
) -> bool {
    central_panel(ctx, egui::Align::Min, |ui| {
        ui.heading("Settings");
        ui.add_space(20.0);
        if ui.button(RichText::new("Display").heading()).clicked() {
            state.menu = SettingsMenu::Display(settings.zoom.clone());
        }
        if ui.button(RichText::new("Keyboard").heading()).clicked() {
            state.menu = SettingsMenu::Keys;
        }
        if ui.button(RichText::new("Help").heading()).clicked() {
            state.menu = SettingsMenu::Help;
        }
        ui.add_space(20.0);
        ui.button(RichText::new("Close").heading()).clicked()
    })
}

pub fn settings_menu_display(
    app: &mut App,
    res: &Resources,
    settings: &mut Settings,
    temp_window: &mut ZoomSettings,
    ctx: &Context,
) -> bool {
    let r = central_panel(ctx, egui::Align::Min, |ui| {
        ui.heading("Display settings");
        ui.add_space(20.0);
        display_settings(app, res, settings, temp_window, ui);
        ui.add_space(20.0);
        ui.button(RichText::new("Back").heading()).clicked()
    });
    ctx.set_theme(settings.ui_theme);
    r
}

pub fn settings_menu_help(settings: &mut Settings, state: &mut SettingsState, ctx: &Context) {
    central_panel(ctx, egui::Align::Min, |ui| {
        ui.heading("Help settings");
        ui.add_space(20.0);
        ui.checkbox(&mut settings.tutorial, "Tutorial");
        ui.checkbox(&mut settings.animate_tooltips, "Animated tooltips");
        ui.add_space(20.0);
        if ui.button(RichText::new("Back").heading()).clicked() {
            state.menu = SettingsMenu::Main;
        }
    });
}

pub fn settings_menu_keys(settings: &mut Settings, state: &mut SettingsState, ctx: &Context) {
    central_panel(ctx, egui::Align::Min, |ui| {
        ui.heading("Keyboard commands");
        Grid::new("key grid").show(ui, |ui| {
            for (n, (k, v)) in settings.keys.iter().enumerate() {
                ui.label(k.name());
                if ui.button(key_name(*v)).clicked() {
                    state.menu = SettingsMenu::ChooseKey(*k);
                }
                if n & 1 != 0 {
                    ui.end_row();
                }
            }
        });
        ui.add_space(20.0);
        if ui.button(RichText::new("Back").heading()).clicked() {
            state.menu = SettingsMenu::Main;
        };
    })
}

pub fn settings_menu_choose_key(
    app: &App,
    settings: &mut Settings,
    ctx: &Context,
    action: &Action,
) -> bool {
    let mut back = false;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Press any key");
        if let Some(key) = app.keyboard.pressed.iter().next() {
            settings.keys.insert(*action, *key);
            back = true;
        }
    });
    back
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
                        banned: Default::default(),
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
