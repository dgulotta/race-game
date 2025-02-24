use notan::egui::{self, Context, Ui};

pub fn central_panel<R>(
    ctx: &Context,
    align: egui::Align,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    egui::CentralPanel::default()
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(align), |ui| {
                egui::ScrollArea::both()
                    .auto_shrink(egui::Vec2b::new(false, false))
                    .show(ui, add_contents)
                    .inner
            })
            .inner
        })
        .inner
}
