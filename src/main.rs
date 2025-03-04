use notan::draw::DrawConfig;
use notan::egui::{EguiConfig, EguiPluginSugar};
use notan::extra::FpsLimit;
use notan::prelude::*;
use race::save::{load_or_log_err, save_or_log_err};
use race::states::SelectState;
use race::ui::loader::Resources;
use race::ui::menu::apply_zoom_settings;
use race::ui::screen::Screen;
use race::ui::settings::Settings;
use takeable::Takeable;

#[derive(AppState)]
struct GameData {
    resources: Resources,
    state: Takeable<Box<dyn Screen>>,
    settings: Settings,
}

fn window_config() -> WindowConfig {
    let config = WindowConfig::new()
        .set_vsync(true)
        .set_title("Race")
        .set_app_id("race");
    if cfg!(target_arch = "wasm32") {
        config.set_maximized(true).set_resizable(true)
    } else {
        let sz: Option<(u32, u32)> = load_or_log_err("window_size", "failed to load window size")
            .unwrap_or(Some((1024, 576)));
        match sz {
            Some((w, h)) => config.set_size(w, h),
            None => config.set_fullscreen(true),
        }
    }
}

#[notan_main]
fn main() -> Result<(), String> {
    let mut builder = notan::init_with(init)
        .add_plugin(FpsLimit::new(30))
        .add_config(window_config())
        .add_config(DrawConfig)
        .add_config(EguiConfig)
        .draw(draw);
    if cfg!(not(target_arch = "wasm32")) {
        builder = builder.event(event);
    }
    builder.build()
}

fn adjust_font_sizes(gfx: &mut Graphics, plugins: &mut Plugins) {
    let factor = 1.3;
    let output = plugins.egui(|ctx| {
        ctx.style_mut(|style| {
            for (_, v) in style.text_styles.iter_mut() {
                v.size *= factor;
            }
        });
    });
    gfx.render(&output);
}

fn event(app: &mut App, event: Event) {
    if matches!(event, Event::Exit) {
        let window_size = if app.window().is_fullscreen() {
            None
        } else {
            Some(app.window().size())
        };
        save_or_log_err("window_size", &window_size, "Failed to save window size");
    }
}

fn init(gfx: &mut Graphics, plugins: &mut Plugins) -> GameData {
    adjust_font_sizes(gfx, plugins);
    let resources = Resources::load_all(gfx);

    let state: Box<dyn Screen> = Box::new(SelectState::new(&resources.levels));
    let settings: Settings =
        load_or_log_err("settings", "failed to load settings").unwrap_or_default();
    let output = plugins.egui(|ctx| apply_zoom_settings(&settings, ctx));
    gfx.render(&output);
    GameData {
        resources,
        state: Takeable::new(state),
        settings,
    }
}

fn draw(app: &mut App, gfx: &mut Graphics, plugins: &mut Plugins, data: &mut GameData) {
    data.state
        .borrow(|st| st.run(app, gfx, plugins, &data.resources, &mut data.settings));
}
