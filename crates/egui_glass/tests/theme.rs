use egui::{Color32, Context, RawInput, Stroke, Visuals};
use egui_glass::{Glass, GlassToolbar};

#[test]
fn containers_preserve_host_visuals_only_when_requested() {
    for dark in [false, true] {
        for toolbar in [false, true] {
            for preserve in [None, Some(false), Some(true)] {
                let ctx = Context::default();
                let _ = ctx.run_ui(RawInput::default(), |ui| {
                    let mut visuals = if dark { Visuals::dark() } else { Visuals::light() };
                    visuals.widgets.inactive.bg_fill = Color32::RED;
                    visuals.widgets.inactive.bg_stroke = Stroke::new(3.0, Color32::GREEN);
                    visuals.selection.bg_fill = Color32::BLUE;
                    ui.style_mut().visuals = visuals.clone();
                    let check = |ui: &mut egui::Ui| {
                        if preserve == Some(true) {
                            assert_eq!(ui.visuals(), &visuals);
                        } else {
                            assert_eq!(ui.visuals().widgets.inactive.bg_fill, Color32::TRANSPARENT);
                            assert_eq!(ui.visuals().widgets.inactive.bg_stroke, Stroke::NONE);
                            assert_ne!(ui.visuals().selection.bg_fill, Color32::BLUE);
                        }
                        let _ = ui.button("Continue");
                        42
                    };
                    let result = if toolbar {
                        let bar = GlassToolbar::default();
                        let bar = match preserve { Some(value) => bar.preserve_theme(value), None => bar };
                        bar.show(ui, check).inner
                    } else {
                        let glass = Glass::default();
                        let glass = match preserve { Some(value) => glass.preserve_theme(value), None => glass };
                        glass.show(ui, check).inner
                    };
                    assert_eq!(result, 42);
                    assert_eq!(ui.visuals(), &visuals, "container visuals must not leak into the parent");
                });
            }
        }
    }
}
