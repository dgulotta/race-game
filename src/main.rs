use notan::draw::DrawConfig;
use notan::egui::EguiConfig;
use notan::extra::FpsLimit;
use notan::prelude::*;
use race_game_rust::save::load_or_log_err;
use race_game_rust::states::SelectState;
use race_game_rust::ui::loader::Resources;
use race_game_rust::ui::screen::Screen;
use race_game_rust::ui::settings::Settings;
use takeable::Takeable;

#[derive(AppState)]
struct GameData {
    resources: Resources,
    state: Takeable<Box<dyn Screen>>,
    settings: Settings,
}

fn window_config() -> WindowConfig {
    WindowConfig::new()
        .set_vsync(true)
        .set_title("Race")
        .set_size(1024, 576)
        .set_app_id("race")
}

#[notan_main]
fn main() -> Result<(), String> {
    notan::init_with(init)
        .add_plugin(FpsLimit::new(30))
        .add_config(window_config())
        .add_config(DrawConfig)
        .add_config(EguiConfig)
        .draw(draw)
        .build()
}

fn adjust_font_sizes(gfx: &mut Graphics, plugins: &mut Plugins) {
    let factor = 1.3;
    use notan::egui::EguiPluginSugar;
    let output = plugins.egui(|ctx| {
        ctx.style_mut(|style| {
            for (_, v) in style.text_styles.iter_mut() {
                v.size *= factor;
            }
        });
    });
    gfx.render(&output);
}

fn init(gfx: &mut Graphics, plugins: &mut Plugins) -> GameData {
    adjust_font_sizes(gfx, plugins);
    let resources = Resources::load_all(gfx);

    let state: Box<dyn Screen> = Box::new(SelectState::new(&resources.levels));
    let settings: Settings =
        load_or_log_err("settings", "failed to load settings").unwrap_or_default();
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
