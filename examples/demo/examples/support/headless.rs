use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::{egui, egui_wgpu, wgpu};
use egui::emath::GuiRounding;
use egui_glass::{GlassStyle, LiveBackdrop, LiveBackdropQuality};

pub struct Harness {
    pub state: egui_wgpu::RenderState,
    pub screen: egui_wgpu::ScreenDescriptor,
    ctx: egui::Context,
    live: LiveBackdrop,
    target: wgpu::Texture,
}

impl Harness {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await?;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await?;
        println!("adapter: {:?}", adapter.get_info());
        let target_format = wgpu::TextureFormat::Rgba8Unorm;
        let renderer = egui_wgpu::Renderer::new(&device, target_format, Default::default());
        let state = egui_wgpu::RenderState {
            adapter, available_adapters: vec![], device, queue, target_format,
            renderer: Arc::new(egui::mutex::RwLock::new(renderer)),
            surface_config: egui_wgpu::SurfaceConfig::LOW_LATENCY,
        };
        egui_glass::init(&state, 1);
        let live = LiveBackdrop::new(&state, None);
        let ctx = egui::Context::default();
        let mut image = egui::ColorImage::filled([64, 64], egui::Color32::WHITE);
        for y in 0..64 {
            for x in 0..64 { image[(x, y)] = COLORS[(y / 32) * 2 + x / 32]; }
        }
        egui_glass::set_backdrop(&ctx, &state, &image);
        let target = target(&state.device, [1280, 720]);
        Ok(Self { state, screen: egui_wgpu::ScreenDescriptor { size_in_pixels: [1280, 720], pixels_per_point: 1.0 }, ctx, live, target })
    }

    pub fn resize(&mut self, size: [u32; 2], ppp: f32) {
        self.screen = egui_wgpu::ScreenDescriptor { size_in_pixels: size, pixels_per_point: ppp };
        self.target = target(&self.state.device, size);
    }

    pub fn frame(&self, quality: Option<LiveBackdropQuality>, glass_count: usize, landmarks: bool) -> Result<[f64; 2], Box<dyn std::error::Error>> {
        let start = Instant::now();
        let size = self.screen.size_in_pixels;
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(size[0] as f32, size[1] as f32) / self.screen.pixels_per_point);
        let mut input = egui::RawInput { screen_rect: Some(rect), ..Default::default() };
        input.viewports.get_mut(&egui::ViewportId::ROOT).expect("default root viewport").native_pixels_per_point = Some(self.screen.pixels_per_point);
        let output = self.ctx.run_ui(input, |ui| {
            let page = |ui: &mut egui::Ui| draw_page(ui, rect, landmarks);
            if let Some(quality) = quality {
                self.live.run_with_quality(ui, egui::Color32::WHITE, quality, page);
            } else {
                page(ui);
            }
            draw_glass(ui, rect, glass_count, landmarks);
        });
        let primitives = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        assert_eq!(self.ctx.viewport_rect(), rect.round_ui());
        assert_eq!(output.pixels_per_point, self.screen.pixels_per_point);
        let layout_ms = start.elapsed().as_secs_f64() * 1000.0;
        let mut renderer = self.state.renderer.write();
        for (id, delta) in &output.textures_delta.set {
            renderer.update_texture(&self.state.device, &self.state.queue, *id, delta);
        }
        let mut encoder = self.state.device.create_command_encoder(&Default::default());
        let commands = renderer.update_buffers(&self.state.device, &self.state.queue, &mut encoder, &primitives, &self.screen);
        {
            let view = self.target.create_view(&Default::default());
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, depth_slice: None, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })], ..Default::default()
            }).forget_lifetime();
            renderer.render(&mut pass, &primitives, &self.screen);
        }
        self.state.queue.submit(commands.into_iter().chain([encoder.finish()]));
        self.wait()?;
        for id in &output.textures_delta.free { renderer.free_texture(id); }
        Ok([layout_ms, start.elapsed().as_secs_f64() * 1000.0])
    }

    pub fn verify_landmarks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let [width, height] = self.screen.size_in_pixels;
        let row_bytes = (width * 4).div_ceil(256) * 256;
        let buffer = self.state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("landmark readback"), size: u64::from(row_bytes) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false,
        });
        let mut encoder = self.state.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(self.target.as_image_copy(), wgpu::TexelCopyBufferInfo {
            buffer: &buffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row_bytes), rows_per_image: Some(height) },
        }, self.target.size());
        self.state.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| { let _ = tx.send(result); });
        self.wait()?;
        rx.recv_timeout(Duration::from_secs(10))??;
        let data = buffer.slice(..).get_mapped_range();
        for (i, expected) in COLORS.iter().enumerate() {
            let x = width * (1 + 2 * (i as u32 % 2)) / 4;
            let y = height * (1 + 2 * (i as u32 / 2)) / 4;
            let offset = (y * row_bytes + x * 4) as usize;
            let pixel = &data[offset..offset + 4];
            assert!(pixel.iter().zip(expected.to_array()).all(|(a, b)| a.abs_diff(b) <= 3), "landmark {i}: {pixel:?} != {expected:?}");
        }
        Ok(())
    }

    fn wait(&self) -> Result<(), wgpu::PollError> {
        self.state.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: Some(Duration::from_secs(10)) })?;
        Ok(())
    }
}

const COLORS: [egui::Color32; 4] = [
    egui::Color32::from_rgb(230, 30, 40), egui::Color32::from_rgb(30, 220, 50),
    egui::Color32::from_rgb(30, 50, 230), egui::Color32::from_rgb(230, 220, 40),
];

fn draw_page(ui: &mut egui::Ui, rect: egui::Rect, landmarks: bool) {
    egui_glass::show_backdrop(ui, rect);
    for (i, color) in COLORS.iter().enumerate() {
        let min = rect.min + egui::vec2((i % 2) as f32 * rect.width() * 0.5, (i / 2) as f32 * rect.height() * 0.5);
        ui.painter().rect_filled(egui::Rect::from_min_size(min, rect.size() * 0.5), 0.0, *color);
    }
    if !landmarks {
        for i in 0..300 {
            let pos = rect.min + egui::vec2((i % 10) as f32 * rect.width() / 10.0, (i / 10) as f32 * 22.0);
            ui.painter().text(pos, egui::Align2::LEFT_TOP, "Live backdrop", egui::FontId::proportional(14.0), egui::Color32::BLACK);
        }
    }
}

fn draw_glass(ui: &egui::Ui, rect: egui::Rect, count: usize, landmarks: bool) {
    if landmarks {
        let style = GlassStyle { corner_radius: 0.0, blur: 0.0, refraction: 0.0, chromatic: 0.0,
            tint: egui::Color32::TRANSPARENT, brightness: 1.0, saturation: 1.0, specular: 0.0,
            border: 0.0, shadow: 0.0, ..GlassStyle::regular() };
        egui_glass::paint_glass(ui, rect.shrink(4.0), &style);
    } else {
        for i in 0..count {
            let min = rect.min + egui::vec2(20.0 + (i % 4) as f32 * 180.0, 20.0 + (i / 4) as f32 * 110.0);
            egui_glass::paint_glass(ui, egui::Rect::from_min_size(min, egui::vec2(160.0, 90.0)), &GlassStyle::regular());
        }
    }
}

fn target(device: &wgpu::Device, size: [u32; 2]) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("headless output"), size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    })
}
