use notan::{
    app::{Graphics, Plugins},
    egui::{self, EguiPluginSugar, RichText},
};

pub fn credits_screen(gfx: &mut Graphics, plugins: &mut Plugins) -> bool {
    let mut close = false;
    let output = plugins.egui(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            for line in include_str!("../../res/credits.txt").lines() {
                if let Some(s) = line.strip_prefix("# ") {
                    ui.label(RichText::new(s).text_style(egui::TextStyle::Heading));
                } else {
                    ui.label(line);
                }
            }
            ui.label("");
            if ui.button("Close").clicked() {
                close = true;
            }
        });
    });
    gfx.render(&output);
    close
}
