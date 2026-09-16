use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, ColorImage, Rect, Vec2};
use egui_glass::{Glass, GlassButton, GlassStyle, GlassToolbar};

const MAX_PHOTO_SIZE: u32 = 1600;
const SIDEBAR_WIDTH: f32 = 210.0;
const PAPER: Color32 = Color32::from_rgb(250, 250, 252);

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

fn main() -> eframe::Result {
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
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        let rs = cc.wgpu_render_state.as_ref().expect("demo requires the wgpu backend");
        egui_glass::init(rs, 1);
        let mut photos: Vec<PathBuf> = std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../asset"))
            .map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "jpg" || e == "png")).collect())
            .unwrap_or_default();
        photos.sort();
        let selected = std::env::var("LG_PHOTO").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        let mut app = Self { style: GlassStyle::regular(), selected, photos, caption: &FALLBACK_CAPTION, last_pane: Rect::NOTHING, scroll_offset: 0.0, portrait: false };
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
                egui_glass::set_backdrop(ctx, rs, &compose_backdrop(&photo, self.portrait));
            }
            Err(err) => eprintln!("failed to load {}: {err}", path.display()),
        }
    }

    fn handle_drop(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if let Some(path) = ctx.input(|i| i.raw.dropped_files.first().and_then(|f| f.path.clone())) {
            self.load_photo(ctx, frame.wgpu_render_state().unwrap(), &path);
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("egui_glass");
        ui.label("Sliders drive the buttons, toolbar and back button. The sidebar uses the fixed flat `GlassStyle::panel()`.");
        ui.label("Sidebar items switch the photo; drop an image onto the window to load your own. Drag the glass panels around.");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (name, preset) in [("Regular", GlassStyle::regular()), ("Clear", GlassStyle::clear()), ("Dark", GlassStyle::dark()), ("Panel", GlassStyle::panel())] {
                if ui.button(name).clicked() {
                    self.style = preset;
                }
            }
        });
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

        // Everything except the floating glass scrolls, so the backdrop moves under the sidebar.
        let mut scroll = egui::ScrollArea::vertical().auto_shrink(false);
        if let Some(offset) = std::env::var("LG_SCROLL").ok().and_then(|v| v.parse().ok()) {
            scroll = scroll.vertical_scroll_offset(offset); // dev knob for screenshots
        }
        // egui only routes the wheel to a ScrollArea when the pointer's topmost layer is the
        // scroll area's own; over the floating glass panels we forward it by hand.
        let over_floating = ui.ctx().input(|i| i.pointer.latest_pos()).is_some_and(|p| {
            pane.contains(p) && ui.ctx().layer_id_at(p).is_some_and(|l| l.order == egui::Order::Middle)
        });
        let wheel = ui.ctx().input(|i| i.smooth_scroll_delta.y);
        if over_floating && wheel != 0.0 {
            self.scroll_offset = (self.scroll_offset - wheel).max(0.0);
            scroll = scroll.vertical_scroll_offset(self.scroll_offset);
        }
        let output = scroll
            .show(ui, |ui| {
                let top = ui.max_rect().min;
                let width = ui.available_width();
                // The photo is half of the composed image along the extension axis.
                let (image_rect, photo_rect, paper) = if portrait {
                    // Tall photo fills the viewport height on the right; the article gets the rest.
                    let scale = pane.height() / size.y;
                    let photo_w = size.x / 2.0 * scale;
                    let column = (width - photo_w).max(200.0);
                    let image = Rect::from_min_size(egui::pos2(top.x + column - photo_w, top.y), size * scale);
                    let photo = Rect::from_min_max(egui::pos2(top.x + column, top.y), image.max);
                    (image, photo, Rect::from_min_max(top, egui::pos2(top.x + column, top.y + 4.0 * pane.height())))
                } else {
                    let scale = width / size.x;
                    let image = Rect::from_min_size(top, size * scale);
                    let photo = Rect::from_min_max(top, egui::pos2(image.max.x, image.center().y));
                    (image, photo, Rect::from_min_max(egui::pos2(top.x, photo.max.y), egui::pos2(image.max.x, top.y + 4.0 * pane.height())))
                };
                ui.painter().rect_filled(Rect::from_min_max(top, egui::pos2(top.x + width, paper.max.y)), 0.0, PAPER);
                egui_glass::show_backdrop_mapped(ui, image_rect, pane, PAPER);
                ui.painter().rect_filled(paper, 0.0, PAPER.gamma_multiply(0.93));

                let (left, column_w, top_space) = if portrait {
                    (24.0, paper.width() - 48.0, 200.0)
                } else {
                    ui.add_space(photo_rect.height());
                    (SIDEBAR_WIDTH + 48.0, width - SIDEBAR_WIDTH - 48.0 - 280.0, 20.0)
                };
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
        let photo_rect = output.inner;

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
            Glass::new(GlassStyle::panel()).inner_margin(egui::Margin::symmetric(14, 16)).show(ui, |ui| {
                ui.set_width(SIDEBAR_WIDTH - 28.0);
                if !portrait {
                    ui.set_min_height(pane.height() - 64.0);
                }
                ui.label(egui::RichText::new("▤").size(18.0));
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
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.handle_drop(ui.ctx(), frame);
        egui::Panel::left("controls").exact_size(280.0).show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
        });
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| self.scene(ui, frame));
    }
}

/// The photo plus an equally sized "background extension" on one side
/// (below for landscape, left for portrait): the photo mirrored across that
/// edge, heavily blurred and faded into paper white, so glass over the
/// content area shows only a soft colour wash.
fn compose_backdrop(photo: &image::RgbaImage, portrait: bool) -> ColorImage {
    use image::imageops::{resize, FilterType};
    let (w, h) = (photo.width(), photo.height());
    let soft = resize(&resize(photo, (w / 24).max(1), (h / 24).max(1), FilterType::Triangle), w, h, FilterType::Triangle);
    let (out_w, out_h) = if portrait { (2 * w, h) } else { (w, 2 * h) };
    let paper = [PAPER.r() as f32, PAPER.g() as f32, PAPER.b() as f32];
    let mut pixels = Vec::with_capacity((out_w * out_h) as usize);
    for y in 0..out_h {
        for x in 0..out_w {
            // Distance into the extension (0 = still on the photo) and the source pixel
            // mirrored across the seam: portrait extension is x < w, landscape is y >= h.
            let (dist, sx, sy) = if portrait {
                (w.saturating_sub(x), (w - 1).saturating_sub(x), y)
            } else {
                (y.saturating_sub(h - 1), x, (2 * h - 1).saturating_sub(y))
            };
            let px = if dist == 0 {
                let p = photo.get_pixel(if portrait { x - w } else { x }, y).0;
                Color32::from_rgb(p[0], p[1], p[2])
            } else {
                let p = soft.get_pixel(sx, sy).0;
                let extent = if portrait { w } else { h } as f32;
                let fade = (dist as f32 / (extent * 0.2)).min(1.0); // 0 at the seam -> 1 after 20 %
                let paper_amount = 0.6 + 0.3 * fade;
                let mix = |c: u8, paper: f32| (c as f32 * (1.0 - paper_amount) + paper * paper_amount) as u8;
                Color32::from_rgb(mix(p[0], paper[0]), mix(p[1], paper[1]), mix(p[2], paper[2]))
            };
            pixels.push(px);
        }
    }
    ColorImage::new([out_w as usize, out_h as usize], pixels)
}
