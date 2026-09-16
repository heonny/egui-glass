//! Live backdrop: your content is laid out a second time in a twin egui
//! context (same memory, no input events) and rendered off screen without the
//! glass, then used as the backdrop for this frame. Glass therefore refracts
//! everything drawn beneath it, text included, with no frame of lag. The cost
//! is one extra layout and draw of that content per frame.

use std::collections::HashMap;
use std::sync::Mutex;

use egui::{ClippedPrimitive, Color32, Context, FontDefinitions, Id, Rect, Shape, Ui, UiBuilder};
use egui_wgpu::{CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};

use crate::renderer::GlassResources;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Screen mapping of this frame's live backdrop, read by the glass widgets.
#[derive(Clone, Copy)]
pub(crate) struct LiveState {
    pub pass_nr: u64,
    pub screen_rect: Rect,
}

fn state_id() -> Id {
    Id::new("egui_glass::live")
}

pub(crate) fn live_state(ctx: &Context) -> Option<LiveState> {
    let state: Option<LiveState> = ctx.data(|d| d.get_temp(state_id()));
    state.filter(|s| s.pass_nr == ctx.cumulative_pass_nr())
}

/// Renders content off screen so glass can refract it. Cheap to clone.
#[derive(Clone)]
pub struct LiveBackdrop {
    twin: Context,
}

impl LiveBackdrop {
    /// `fonts` should match what the app installed with `Context::set_fonts`,
    /// so the twin lays text out identically. Create it before [`crate::set_backdrop`]
    /// and before registering images with [`crate::register_native_texture`], so the
    /// off-screen pass can draw them too.
    pub fn new(render_state: &RenderState, fonts: Option<FontDefinitions>) -> Self {
        let twin = Context::default();
        if let Some(fonts) = fonts {
            twin.set_fonts(fonts);
        }
        let resources = LiveResources::new(&render_state.device);
        render_state.renderer.write().callback_resources.insert(resources);
        Self { twin }
    }

    pub fn set_fonts(&self, fonts: FontDefinitions) {
        self.twin.set_fonts(fonts);
    }

    /// Shows `add_contents` in `ui` as usual and, this frame only, makes an
    /// off-screen copy of it the backdrop of every glass surface. Draw your
    /// glass outside of `add_contents` (floating areas, later widgets).
    pub fn run(&self, ui: &mut Ui, clear: Color32, mut add_contents: impl FnMut(&mut Ui)) {
        let ctx = ui.ctx().clone();
        let slot = ui.painter().add(Shape::Noop);
        add_contents(ui);

        // Twin pass: identical memory, identical ui id/rect, no events.
        let mut input = ctx.input(|i| i.raw.clone());
        input.events.clear();
        input.dropped_files.clear();
        input.hovered_files.clear();
        self.twin.memory_mut(|m| *m = ctx.memory(|m| m.clone()));
        let (id, rect, clip, layer, layout) = (ui.id(), ui.max_rect(), ui.clip_rect(), ui.layer_id(), *ui.layout());
        let twin = self.twin.clone();
        let output = self.twin.run_ui(input, |_root| {
            let mut ui = Ui::new(twin.clone(), id, UiBuilder::new().layer_id(layer).max_rect(rect).layout(layout));
            ui.set_clip_rect(clip);
            add_contents(&mut ui);
        });
        let primitives = self.twin.tessellate(output.shapes, output.pixels_per_point);

        let frame = FrameData { primitives, textures_delta: output.textures_delta, clear };
        let callback = LiveCallback { frame: Mutex::new(Some(frame)) };
        ui.painter().set(slot, Shape::Callback(egui_wgpu::Callback::new_paint_callback(rect, callback)));
        let state = LiveState { pass_nr: ctx.cumulative_pass_nr(), screen_rect: ctx.viewport_rect() };
        ctx.data_mut(|d| d.insert_temp(state_id(), state));
    }
}

struct FrameData {
    primitives: Vec<ClippedPrimitive>,
    textures_delta: egui::TexturesDelta,
    clear: Color32,
}

struct LiveCallback {
    frame: Mutex<Option<FrameData>>,
}

impl CallbackTrait for LiveCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(frame) = self.frame.lock().ok().and_then(|mut f| f.take()) else { return Vec::new() };
        let Some(res) = resources.get_mut::<LiveResources>() else { return Vec::new() };
        let mut frame = frame;
        // Native textures live in the app's renderer under different ids; remap the ones we know.
        for primitive in &mut frame.primitives {
            if let egui::epaint::Primitive::Mesh(mesh) = &mut primitive.primitive {
                if let Some(id) = res.texture_map.get(&mesh.texture_id) {
                    mesh.texture_id = *id;
                }
            }
        }
        let [w, h] = screen.size_in_pixels;
        if w == 0 || h == 0 {
            return Vec::new();
        }
        res.ensure_target(device, w, h);
        let target = res.target.as_ref().expect("target created above");

        for (id, delta) in &frame.textures_delta.set {
            res.renderer.update_texture(device, queue, *id, delta);
        }
        let commands = res.renderer.update_buffers(device, queue, encoder, &frame.primitives, screen);
        {
            let clear = egui::Rgba::from(frame.clear);
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui_glass_live"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target.mip_views[0],
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: clear.r() as f64, g: clear.g() as f64, b: clear.b() as f64, a: 1.0 }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                })
                .forget_lifetime();
            res.renderer.render(&mut pass, &frame.primitives, screen);
        }
        for id in &frame.textures_delta.free {
            res.renderer.free_texture(id);
        }
        for level in 1..target.mip_views.len() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_glass_mip"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.mip_views[level],
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&res.mip_pipeline);
            pass.set_bind_group(0, &target.mip_bind_groups[level - 1], &[]);
            pass.draw(0..3, 0..1);
        }
        let (sample_view, mips) = (target.sample_view.clone(), target.mip_views.len() as u32);
        let pass_nr = res.pass_counter;
        res.pass_counter += 1;
        if let Some(glass) = resources.get_mut::<GlassResources>() {
            glass.set_live_backdrop(sample_view, mips, pass_nr);
        }
        commands
    }

    fn paint(&self, _info: egui::PaintCallbackInfo, _pass: &mut wgpu::RenderPass<'static>, _resources: &CallbackResources) {}
}

struct Target {
    size: [u32; 2],
    /// One view per mip level, for rendering.
    mip_views: Vec<wgpu::TextureView>,
    /// Non-sRGB view over all levels, sampled by the glass shader as gamma values.
    sample_view: wgpu::TextureView,
    mip_bind_groups: Vec<wgpu::BindGroup>,
}

pub(crate) struct LiveResources {
    renderer: egui_wgpu::Renderer,
    /// App renderer texture id -> id of the same texture in the twin renderer.
    texture_map: HashMap<egui::TextureId, egui::TextureId>,
    mip_pipeline: wgpu::RenderPipeline,
    mip_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    target: Option<Target>,
    /// Counts live frames; glass resources use it to know the live view is current.
    pass_counter: u64,
}

impl LiveResources {
    fn new(device: &wgpu::Device) -> Self {
        let renderer = egui_wgpu::Renderer::new(device, FORMAT, egui_wgpu::RendererOptions::default());
        let shader = device.create_shader_module(wgpu::include_wgsl!("mipgen.wgsl"));
        let mip_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("egui_glass_mip"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("egui_glass_mip"),
            bind_group_layouts: &[Some(&mip_layout)],
            immediate_size: 0,
        });
        let mip_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("egui_glass_mip"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { format: FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("egui_glass_mip"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self { renderer, texture_map: HashMap::new(), mip_pipeline, mip_layout, sampler, target: None, pass_counter: 0 }
    }

    /// Mirrors a native texture of the app's renderer (`app_id`) into the twin renderer.
    pub(crate) fn register_native_texture(&mut self, device: &wgpu::Device, view: &wgpu::TextureView, app_id: egui::TextureId) {
        let id = self.renderer.register_native_texture(device, view, wgpu::FilterMode::Linear);
        self.texture_map.insert(app_id, id);
    }

    pub(crate) fn free_texture(&mut self, app_id: &egui::TextureId) {
        if let Some(id) = self.texture_map.remove(app_id) {
            self.renderer.free_texture(&id);
        }
    }

    fn ensure_target(&mut self, device: &wgpu::Device, w: u32, h: u32) {
        if self.target.as_ref().is_some_and(|t| t.size == [w, h]) {
            return;
        }
        let mip_level_count = 32 - w.max(h).leading_zeros();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("egui_glass_live"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        let mip_views: Vec<_> = (0..mip_level_count)
            .map(|level| {
                texture.create_view(&wgpu::TextureViewDescriptor { base_mip_level: level, mip_level_count: Some(1), ..Default::default() })
            })
            .collect();
        let mip_bind_groups = mip_views[..mip_views.len() - 1]
            .iter()
            .map(|view| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("egui_glass_mip"),
                    layout: &self.mip_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(view) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                    ],
                })
            })
            .collect();
        let sample_view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        });
        self.target = Some(Target { size: [w, h], mip_views, sample_view, mip_bind_groups });
    }
}
