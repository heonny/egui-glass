mod fonts;

use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, ColorImage, Rect, Vec2};
use egui_glass::{Glass, GlassButton, GlassStyle, GlassToolbar, LiveBackdrop};

const MAX_PHOTO_SIZE: u32 = 1600;
const SIDEBAR_WIDTH: f32 = 210.0;
/// Page colours: macOS/iOS light and dark system backgrounds.
const PAPER: Color32 = Color32::from_rgb(250, 250, 252);
const PAPER_DARK: Color32 = Color32::from_rgb(28, 28, 30);

/// Text shown for a bundled photo, matched by a substring of its file name.
struct Caption {
    file_key: &'static str,
    item: &'static str,
    title: &'static str,
    subtitle: &'static str,
    body: [&'static str; 2],
}

const CAPTIONS: [Caption; 3] = [
    Caption {
        file_key: "Parasol",
        item: "Parasol",
        title: "Woman with a Parasol",
        subtitle: "Claude Monet, 1875. Oil on canvas. National Gallery of Art, Washington.",
        body: [
            "Camille Monet and the couple's son Jean stand on a windswept rise near Argenteuil, seen from below \
             against a sky of fast-moving clouds. The low viewpoint and the flurry of short strokes in the grass \
             and the veil make the wind almost visible.",
            "Monet is said to have painted it in a single session. It is one of the clearest statements of the \
             early Impressionist aim: not the figures themselves, but the light and air around them.",
        ],
    },
    Caption {
        file_key: "Studio_Boat",
        item: "Studio Boat",
        title: "The Studio Boat",
        subtitle: "Claude Monet, 1876. Oil on canvas. Barnes Foundation, Philadelphia.",
        body: [
            "Monet had a small boat fitted with a cabin so he could paint from the middle of the Seine, drifting \
             among the reflections instead of watching them from the bank. Here he paints the floating studio \
             itself, moored in the still water at Argenteuil.",
            "The green hull and its mirror image are built from the same broken touches as the water, so the \
             boat seems to dissolve into the river it was made to observe.",
        ],
    },
    Caption {
        file_key: "Argenteuil",
        item: "Argenteuil",
        title: "The Bridge at Argenteuil",
        subtitle: "Claude Monet, 1874. Oil on canvas. Musée d'Orsay, Paris.",
        body: [
            "The road bridge over the Seine at Argenteuil, with sailing boats resting in the calm water of the \
             boating basin that made the town a favourite weekend escape from Paris.",
            "Painted the year of the first Impressionist exhibition, the picture pairs the crisp geometry of the \
             bridge with a surface of water where every mast and arch is echoed in loose, wavering strokes.",
        ],
    },
];

const FALLBACK_CAPTION: Caption = Caption {
    file_key: "",
    item: "Photo",
    title: "Your photo",
    subtitle: "Dropped onto the window.",
    body: ["Glass surfaces refract whatever backdrop you register.", "Drag the panels around to compare the presets."],
};

fn caption_for(path: &Path) -> &'static Caption {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    CAPTIONS.iter().find(|c| name.contains(c.file_key)).unwrap_or(&FALLBACK_CAPTION)
}

/// Minimal stderr logger so wgpu/egui warnings (e.g. validation errors) are visible.
struct StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Warn
    }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            eprintln!("[{}] {}: {}", record.level(), record.target(), record.args());
        }
    }
    fn flush(&self) {}
}

fn main() -> eframe::Result {
    let _ = log::set_logger(&StderrLogger).map(|()| log::set_max_level(log::LevelFilter::Warn));
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 800.0]).with_title("egui_glass demo"),
        ..Default::default()
    };
    eframe::run_native("egui_glass_demo", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

struct App {
    style: GlassStyle,
    selected: usize,
    photos: Vec<PathBuf>,
    /// Theme the visuals were last built for.
    dark: bool,
    live: LiveBackdrop,
    /// Render the page off screen too, so glass refracts text as well as the photo.
    live_mode: bool,
    caption: &'static Caption,
    /// Pane rect of the previous frame; floating glass is re-anchored when it changes.
    last_pane: Rect,
    /// Vertical scroll offset of the page, kept so wheel input over floating glass can drive it.
    scroll_offset: f32,
    /// Whether the current backdrop was composed for a portrait photo
    /// (extension to the left) or a landscape one (extension below).
    portrait: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_fonts(fonts::system_fonts());
        let rs = cc.wgpu_render_state.as_ref().expect("demo requires the wgpu backend");
        egui_glass::init(rs, 1);
        let live = LiveBackdrop::new(rs, Some(fonts::system_fonts()));
        let mut photos: Vec<PathBuf> = std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../asset"))
            .map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "jpg" || e == "png")).collect())
            .unwrap_or_default();
        photos.sort();
        let selected = std::env::var("LG_PHOTO").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        let style = match std::env::var("LG_PRESET").as_deref() {
            Ok("dark") => GlassStyle::dark(),
            Ok("clear") => GlassStyle::clear(),
            _ => GlassStyle::regular(),
        };
        let mut app = Self { style, selected, photos, dark: style.is_dark(), live, live_mode: std::env::var_os("LG_LIVE").is_none_or(|v| v != "0"), caption: &FALLBACK_CAPTION, last_pane: Rect::NOTHING, scroll_offset: 0.0, portrait: false };
        app.apply_visuals(&cc.egui_ctx);
        if let Some(path) = app.photos.get(selected).cloned() {
            app.load_photo(&cc.egui_ctx, rs, &path);
        }
        app
    }

    fn load_photo(&mut self, ctx: &egui::Context, rs: &eframe::egui_wgpu::RenderState, path: &Path) {
        match image::open(path) {
            Ok(img) => {
                let photo = img.thumbnail(MAX_PHOTO_SIZE, MAX_PHOTO_SIZE).to_rgba8();
                self.portrait = photo.height() > photo.width();
                self.caption = caption_for(path);
                let image = ColorImage::from_rgba_unmultiplied([photo.width() as usize, photo.height() as usize], &photo);
                egui_glass::set_backdrop(ctx, rs, &image);
            }
            Err(err) => eprintln!("failed to load {}: {err}", path.display()),
        }
    }

    fn paper(&self) -> Color32 {
        if self.dark { PAPER_DARK } else { PAPER }
    }

    /// Light or dark theme following the glass tint, with macOS-like colours.
    /// The theme is forced (not the system one) so `set_visuals_of` lands on the theme in use.
    fn apply_visuals(&self, ctx: &egui::Context) {
        let (theme, mut visuals) = if self.dark {
            (egui::Theme::Dark, egui::Visuals::dark())
        } else {
            (egui::Theme::Light, egui::Visuals::light())
        };
        if self.dark {
            visuals.panel_fill = PAPER_DARK;
            visuals.window_fill = PAPER_DARK;
            visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(229, 229, 234);
        }
        ctx.set_theme(theme);
        ctx.set_visuals_of(theme, visuals);
    }

    /// Flip the theme when the glass tint crosses from light to dark or back.
    fn sync_theme(&mut self, ctx: &egui::Context) {
        if self.dark != self.style.is_dark() {
            self.dark = self.style.is_dark();
            self.apply_visuals(ctx);
        }
    }

    fn handle_drop(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if let Some(path) = ctx.input(|i| i.raw.dropped_files.first().and_then(|f| f.path.clone())) {
            self.load_photo(ctx, frame.wgpu_render_state().unwrap(), &path);
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("egui_glass");
        ui.label("Sliders drive the buttons, toolbar and back button. The sidebar always uses the flat panel preset; a dark tint switches the whole page to dark mode.");
        ui.label("Sidebar items switch the photo; drop an image onto the window to load your own. Drag the glass panels around.");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (name, preset) in [("Regular", GlassStyle::regular()), ("Clear", GlassStyle::clear()), ("Dark", GlassStyle::dark()), ("Panel", GlassStyle::panel())] {
                if ui.button(name).clicked() {
                    self.style = preset;
                }
            }
        });
        ui.checkbox(&mut self.live_mode, "Live backdrop (glass refracts text too)");
        ui.separator();
        let s = &mut self.style;
        let slider = |ui: &mut egui::Ui, v: &mut f32, range: std::ops::RangeInclusive<f32>, label: &str| {
            ui.add(egui::Slider::new(v, range).text(label));
        };
        ui.label("Shape");
        slider(ui, &mut s.corner_radius, 0.0..=80.0, "corner radius");
        slider(ui, &mut s.corner_smoothing, 0.0..=1.0, "corner smoothing");
        ui.label("Lens");
        slider(ui, &mut s.refraction, 0.0..=60.0, "refraction");
        slider(ui, &mut s.edge_width, 1.0..=120.0, "edge width");
        slider(ui, &mut s.chromatic, 0.0..=1.0, "chromatic");
        ui.label("Material");
        slider(ui, &mut s.blur, 0.0..=80.0, "blur");
        ui.horizontal(|ui| {
            ui.color_edit_button_srgba(&mut s.tint);
            ui.label("tint (alpha = amount)");
        });
        slider(ui, &mut s.brightness, 0.5..=1.6, "brightness");
        slider(ui, &mut s.saturation, 0.0..=2.0, "saturation");
        ui.label("Light");
        slider(ui, &mut s.specular, 0.0..=1.0, "specular");
        slider(ui, &mut s.border, 0.0..=1.0, "border");
        slider(ui, &mut s.shadow, 0.0..=1.0, "shadow");
        slider(ui, &mut s.shadow_radius, 0.0..=60.0, "shadow radius");
    }

    fn scene(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame) {
        let pane = ui.max_rect();
        let Some(size) = egui_glass::backdrop_size(ui.ctx()).filter(|s| s.x > 0.0 && s.y > 0.0) else { return };
        let portrait = self.portrait;
        let paper_color = self.paper();

        // egui only routes the wheel to a ScrollArea when the pointer's topmost layer is the
        // scroll area's own; over the floating glass panels we forward it by hand.
        let over_floating = ui.ctx().input(|i| i.pointer.latest_pos()).is_some_and(|p| {
            pane.contains(p) && ui.ctx().layer_id_at(p).is_some_and(|l| l.order == egui::Order::Middle)
        });
        let wheel = ui.ctx().input(|i| i.smooth_scroll_delta.y);
        let mut forced_offset = std::env::var("LG_SCROLL").ok().and_then(|v| v.parse().ok()); // dev knob for screenshots
        if over_floating && wheel != 0.0 {
            self.scroll_offset = (self.scroll_offset - wheel).max(0.0);
            forced_offset = Some(self.scroll_offset);
        }

        // The page (photo + article) scrolls under the floating glass. In live mode it is also
        // rendered off screen so the glass refracts the text.
        let live = self.live_mode.then(|| self.live.clone());
        let mut photo_rect = Rect::NOTHING;
        let mut page = |ui: &mut egui::Ui| photo_rect = self.page(ui, pane, size, paper_color, forced_offset);
        match live {
            Some(live) => live.run(ui, paper_color, page),
            None => page(ui),
        }

        let style = self.style;
        let relayout = self.last_pane != pane;
        self.last_pane = pane;
        let area = |id: &str, pos: egui::Pos2| {
            // constrain(false): on an area's first (sizing) frame egui assumes a 600x400 size and
            // would clamp the position into the screen and store that wrong position.
            let area = egui::Area::new(egui::Id::new((id, portrait))).movable(true).constrain(false).default_pos(pos);
            if relayout { area.current_pos(pos) } else { area }
        };

        area("sidebar", pane.min + Vec2::splat(16.0)).show(ui.ctx(), |ui| {
            ui.set_width(SIDEBAR_WIDTH);
            Glass::new(if self.dark { GlassStyle::panel_dark() } else { GlassStyle::panel() }).inner_margin(egui::Margin::symmetric(14, 16)).show(ui, |ui| {
                ui.set_width(SIDEBAR_WIDTH - 28.0);
                if !portrait {
                    ui.set_min_height(pane.height() - 64.0);
                }
                sidebar_icon(ui);
                ui.add_space(10.0);
                for i in 0..self.photos.len() {
                    let item = caption_for(&self.photos[i]).item;
                    if ui.selectable_label(self.selected == i, format!("🖼  {item}")).clicked() && self.selected != i {
                        self.selected = i;
                        let path = self.photos[i].clone();
                        self.load_photo(ui.ctx(), frame.wgpu_render_state().unwrap(), &path);
                    }
                }
            });
        });

        let back_pos = if portrait { egui::pos2(photo_rect.min.x, pane.min.y) + Vec2::splat(16.0) } else { pane.min + Vec2::new(SIDEBAR_WIDTH + 32.0, 16.0) };
        area("back", back_pos).show(ui.ctx(), |ui| {
            GlassButton::new(egui::RichText::new("‹").size(22.0)).icon().style(style).show(ui);
        });

        area("toolbar", pane.max - Vec2::splat(20.0)).pivot(egui::Align2::RIGHT_BOTTOM).show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                GlassToolbar::new(style).show(ui, |ui| {
                    for icon in ["↩", "🗀", "🗑"] {
                        let _ = ui.button(egui::RichText::new(icon).size(18.0));
                    }
                });
                ui.add_space(8.0);
                GlassButton::new(egui::RichText::new("✏").size(18.0)).icon().style(style).show(ui);
            });
        });
    }

    /// The scrolling page: photo, paper and article. Returns the photo's screen rect.
    fn page(&mut self, ui: &mut egui::Ui, pane: Rect, size: Vec2, paper_color: Color32, forced_offset: Option<f32>) -> Rect {
        let portrait = self.portrait;
        let mut scroll = egui::ScrollArea::vertical().auto_shrink(false);
        if let Some(offset) = forced_offset {
            scroll = scroll.vertical_scroll_offset(offset);
        }
        let output = scroll.show(ui, |ui| {
            let top = ui.max_rect().min;
            let width = ui.available_width();
            // Everything that is not the photo is plain paper, on screen and for the glass alike.
            let photo_rect = if portrait {
                // Tall photo fills the viewport height on the right; the article gets the rest.
                let scale = pane.height() / size.y;
                let column = (width - size.x * scale).max(200.0);
                Rect::from_min_size(egui::pos2(top.x + column, top.y), size * scale)
            } else {
                Rect::from_min_size(top, size * (width / size.x))
            };
            ui.painter().rect_filled(Rect::from_min_size(top, Vec2::new(width, 4.0 * pane.height())), 0.0, paper_color);
            egui_glass::show_backdrop_mapped(ui, photo_rect, pane, paper_color);

            let (left, column_w, top_space) = if portrait {
                (24.0, photo_rect.min.x - top.x - 48.0, 200.0)
            } else {
                ui.add_space(photo_rect.height());
                (SIDEBAR_WIDTH + 48.0, width - SIDEBAR_WIDTH - 48.0 - 280.0, 20.0)
            };
            let column_w = column_w.max(120.0); // narrow windows: keep the layout valid
            ui.horizontal(|ui| {
                ui.add_space(left);
                ui.vertical(|ui| {
                    ui.set_max_width(column_w);
                    ui.add_space(top_space);
                    self.article(ui);
                });
            });
            let end = photo_rect.max.y.max(ui.cursor().top()) + pane.height() - 200.0;
            ui.add_space((end - ui.cursor().top()).max(0.0));
            photo_rect
        });
        self.scroll_offset = output.state.offset.y;
        output.inner
    }

    /// Article text for the current photo, laid out in the paper-white content area.
    fn article(&self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new(self.caption.title).size(26.0).strong());
        ui.label(egui::RichText::new(self.caption.subtitle).weak());
        ui.add_space(8.0);
        for paragraph in self.caption.body {
            ui.label(paragraph);
            ui.add_space(10.0);
        }
    }
}

impl eframe::App for App {
    /// Keep the floating panels at their designed positions between runs.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.sync_theme(ui.ctx());
        self.handle_drop(ui.ctx(), frame);
        egui::Panel::left("controls").exact_size(280.0).show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
        });
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| self.scene(ui, frame));
    }
}

/// Apple-style "toggle sidebar" glyph: a rounded rectangle with a divider.
fn sidebar_icon(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(20.0, 16.0), egui::Sense::hover());
    let stroke = egui::Stroke::new(1.5, ui.visuals().strong_text_color());
    ui.painter().rect_stroke(rect, 3.0, stroke, egui::StrokeKind::Inside);
    let x = rect.min.x + 7.0;
    ui.painter().line_segment([egui::pos2(x, rect.min.y + 1.0), egui::pos2(x, rect.max.y - 1.0)], stroke);
}
