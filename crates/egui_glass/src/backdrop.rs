use egui::{Color32, ColorImage, Context, Id, Rect, TextureId, Ui, Vec2};
use egui_wgpu::RenderState;

use crate::live::LiveResources;
use crate::renderer::GlassResources;

/// Where the backdrop image is mapped on screen. Stored in egui memory so
/// widgets can find it without touching wgpu.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BackdropState {
    pub texture: TextureId,
    pub size: Vec2,
    /// Screen rect (points) the *whole* image is mapped to.
    pub rect: Rect,
    /// Colour glass shows where it samples outside `rect`.
    pub fill: Color32,
}

fn state_id() -> Id {
    Id::new("egui_glass::backdrop")
}

pub(crate) fn backdrop_state(ctx: &Context) -> Option<BackdropState> {
    ctx.data(|d| d.get_temp(state_id()))
}

/// Screen rect (points) the whole backdrop image is currently mapped to, as
/// recorded by the last [`show_backdrop`]. Useful to align content with it.
pub fn backdrop_rect(ctx: &Context) -> Option<Rect> {
    backdrop_state(ctx).map(|s| s.rect)
}

/// Uploads `image` as the backdrop every glass surface refracts, replacing any
/// previous one. Returns the egui texture id so the app can draw it.
pub fn set_backdrop(ctx: &Context, render_state: &RenderState, image: &ColorImage) -> TextureId {
    let size = image.size;
    let mips = cpu_mip_chain(image);
    let texture = render_state.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("egui_glass_backdrop"),
        size: wgpu::Extent3d { width: size[0] as u32, height: size[1] as u32, depth_or_array_layers: 1 },
        mip_level_count: mips.len() as u32,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (level, (w, h, pixels)) in mips.iter().enumerate() {
        render_state.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: level as u32,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(*h) },
            wgpu::Extent3d { width: *w, height: *h, depth_or_array_layers: 1 },
        );
    }
    let view = texture.create_view(&Default::default());

    let mut renderer = render_state.renderer.write();
    if let Some(previous) = backdrop_state(ctx) {
        free_native_texture(&mut renderer, &previous.texture);
    }
    let texture_id = register_native_texture_in(&mut renderer, &render_state.device, &view);
    let resources: &mut GlassResources = renderer
        .callback_resources
        .get_mut()
        .expect("egui_glass::init must be called before set_backdrop");
    resources.set_backdrop(&render_state.device, view, mips.len() as u32);
    drop(renderer);

    let state = BackdropState {
        texture: texture_id,
        size: Vec2::new(size[0] as f32, size[1] as f32),
        rect: Rect::ZERO,
        fill: Color32::TRANSPARENT,
    };
    ctx.data_mut(|d| d.insert_temp(state_id(), state));
    texture_id
}

/// Registers a wgpu texture for drawing with egui, in the app's renderer and
/// (if a [`crate::LiveBackdrop`] exists) in its off-screen renderer too, so
/// images drawn with it also show through live glass.
pub fn register_native_texture(render_state: &RenderState, view: &wgpu::TextureView) -> TextureId {
    register_native_texture_in(&mut render_state.renderer.write(), &render_state.device, view)
}

/// Frees a texture registered with [`register_native_texture`].
pub fn free_native_texture_id(render_state: &RenderState, id: &TextureId) {
    free_native_texture(&mut render_state.renderer.write(), id);
}

fn register_native_texture_in(renderer: &mut egui_wgpu::Renderer, device: &wgpu::Device, view: &wgpu::TextureView) -> TextureId {
    let id = renderer.register_native_texture(device, view, wgpu::FilterMode::Linear);
    if let Some(live) = renderer.callback_resources.get_mut::<LiveResources>() {
        live.register_native_texture(device, view, id);
    }
    id
}

fn free_native_texture(renderer: &mut egui_wgpu::Renderer, id: &TextureId) {
    renderer.free_texture(id);
    if let Some(live) = renderer.callback_resources.get_mut::<LiveResources>() {
        live.free_texture(id);
    }
}

/// Draws the backdrop image covering `rect` (aspect-fill, centered) and records
/// the mapping so glass widgets sample the same pixels.
pub fn show_backdrop(ui: &mut Ui, rect: Rect) {
    let Some(state) = backdrop_state(ui.ctx()) else { return };
    let scale = (rect.width() / state.size.x).max(rect.height() / state.size.y);
    let fill = ui.visuals().panel_fill;
    show_backdrop_mapped(ui, Rect::from_center_size(rect.center(), state.size * scale), rect, fill);
}

/// Draws the whole backdrop image into `image_rect` (clipped to `clip`) and
/// records that mapping. Use this when you place the image yourself. Glass that
/// samples outside `image_rect` shows `outside` (typically your page colour).
pub fn show_backdrop_mapped(ui: &mut Ui, image_rect: Rect, clip: Rect, outside: Color32) {
    let Some(mut state) = backdrop_state(ui.ctx()) else { return };
    state.rect = image_rect;
    state.fill = outside;
    ui.ctx().data_mut(|d| d.insert_temp(state_id(), state));
    let painter = ui.painter().with_clip_rect(clip);
    painter.image(state.texture, image_rect, Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
}

/// Pixel size of the registered backdrop image.
pub fn backdrop_size(ctx: &Context) -> Option<Vec2> {
    backdrop_state(ctx).map(|s| s.size)
}

/// Box-filtered mip chain: (width, height, rgba bytes) per level.
/// Colour channels are averaged in linear light (sRGB decoded via LUT and
/// re-encoded), so blurred levels keep the brightness of the original.
fn cpu_mip_chain(image: &ColorImage) -> Vec<(u32, u32, Vec<u8>)> {
    let (mut w, mut h) = (image.size[0] as u32, image.size[1] as u32);
    if w == 0 || h == 0 {
        return vec![(1, 1, vec![0; 4])];
    }
    let decode: Vec<f32> = (0..256).map(|i| srgb_to_linear(i as f32 / 255.0)).collect();
    let encode: Vec<u8> = (0..=ENCODE_STEPS).map(|i| (linear_to_srgb(i as f32 / ENCODE_STEPS as f32) * 255.0).round() as u8).collect();
    let mut cur: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    let mut levels = vec![(w, h, cur.clone())];
    while w > 1 || h > 1 {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut next = vec![0u8; (nw * nh * 4) as usize];
        for y in 0..nh {
            for x in 0..nw {
                let (x0, y0) = (x * 2, y * 2);
                let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
                let taps = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];
                for c in 0..3 {
                    let sum: f32 = taps.iter().map(|&(sx, sy)| decode[cur[((sy * w + sx) * 4 + c) as usize] as usize]).sum();
                    next[((y * nw + x) * 4 + c) as usize] = encode[(sum / 4.0 * ENCODE_STEPS as f32).round() as usize];
                }
                let alpha: u32 = taps.iter().map(|&(sx, sy)| cur[((sy * w + sx) * 4 + 3) as usize] as u32).sum();
                next[((y * nw + x) * 4 + 3) as usize] = ((alpha + 2) / 4) as u8;
            }
        }
        w = nw;
        h = nh;
        cur = next;
        levels.push((w, h, cur.clone()));
    }
    levels
}

const ENCODE_STEPS: usize = 4095;

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_image_yields_placeholder_level() {
        let mips = cpu_mip_chain(&ColorImage::new([0, 0], vec![]));
        assert_eq!(mips, vec![(1, 1, vec![0; 4])]);
    }

    #[test]
    fn mip_chain_reaches_one_pixel_average() {
        let img = ColorImage::new([4, 2], vec![egui::Color32::from_rgb(0, 0, 0); 4]
            .into_iter()
            .chain(vec![egui::Color32::from_rgb(255, 255, 255); 4])
            .collect());
        let mips = cpu_mip_chain(&img);
        assert_eq!(mips.len(), 3); // 4x2 -> 2x1 -> 1x1
        // Linear-light average of black and white is 0.5, which encodes to ~188, not 128.
        assert!((187..=188).contains(&mips[2].2[0]), "got {}", mips[2].2[0]);
        assert_eq!(mips[2].2[3], 255);
    }

    #[test]
    fn srgb_round_trip_is_identity() {
        for i in 0..=255u8 {
            let back = (linear_to_srgb(srgb_to_linear(i as f32 / 255.0)) * 255.0).round() as u8;
            assert_eq!(back, i);
        }
    }
}
