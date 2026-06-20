//! tt-splat-viewer — a from-scratch Rust + wgpu viewer for the tt-splat poly-splat + WSR
//! Gaussian-splatting model. See `CLAUDE.md` for the full design + math + build plan.
//!
//! Pipeline (CLAUDE.md §2): CPU preprocess (project gaussians → 2D mean + conic) → additive
//! poly+WSR quads into an `Rgba16Float` accumulator → one fullscreen pass divides
//! `C = (Σ w·color + w_b·c_b) / (Σ w + w_b)`. No depth sort, no transcendentals in the hot path.
//!
//! NOTE: as of this commit the renderer has not been built/run (no native toolchain on the dev box
//! yet) — it is written against the oracle math and the wgpu 0.20 API, pending a build + PSNR check.

pub mod scene;

#[cfg(not(target_arch = "wasm32"))]
pub mod offscreen;

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use scene::{Background, InstanceRaw};

const ACCUM_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// `globals` uniform for the splat pass (16-byte aligned).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GlobalsRaw {
    viewport: [f32; 2],
    _pad: [f32; 2],
}

/// `comp` uniform for the composite pass (vec3 + f32 = 16 bytes).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CompositeRaw {
    c_b: [f32; 3],
    w_b: f32,
}

/// GPU resources for the two-pass WSR renderer.
struct Renderer {
    splat_pipeline: wgpu::RenderPipeline,
    splat_bind_group: wgpu::BindGroup,
    globals_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    instance_count: u32,

    composite_pipeline: wgpu::RenderPipeline,
    composite_bgl: wgpu::BindGroupLayout,
    composite_bind_group: wgpu::BindGroup,
    comp_buf: wgpu::Buffer,

    accum_view: wgpu::TextureView,
}

impl Renderer {
    fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        instances: &[InstanceRaw],
        bg: &Background,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("splat+composite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // ----- splat pass -----
        let globals_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globals"),
            contents: bytemuck::bytes_of(&GlobalsRaw {
                viewport: [width as f32, height as f32],
                _pad: [0.0, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instances"),
            contents: bytemuck::cast_slice(instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let splat_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("splat globals bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let splat_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("splat globals bg"),
            layout: &splat_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buf.as_entire_binding(),
            }],
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2, // mu2d
                1 => Float32x2, // half_extent
                2 => Float32x3, // conic
                3 => Float32,   // opacity
                4 => Float32x3, // color
            ],
        };

        let splat_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("splat layout"),
            bind_group_layouts: &[&splat_bgl],
            push_constant_ranges: &[],
        });

        let additive = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        };
        let splat_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("splat pipeline"),
            layout: Some(&splat_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_splat",
                buffers: &[instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_splat",
                targets: &[Some(wgpu::ColorTargetState {
                    format: ACCUM_FORMAT,
                    blend: Some(wgpu::BlendState {
                        color: additive,
                        alpha: additive,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // ----- composite pass -----
        let comp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("composite uniform"),
            contents: bytemuck::bytes_of(&CompositeRaw {
                c_b: bg.c_b.to_array(),
                w_b: bg.w_b,
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let composite_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("composite bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("composite layout"),
            bind_group_layouts: &[&composite_bgl],
            push_constant_ranges: &[],
        });
        let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("composite pipeline"),
            layout: Some(&composite_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_composite",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_composite",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let accum_view = make_accum(device, width, height);
        let composite_bind_group =
            make_composite_bg(device, &composite_bgl, &accum_view, &comp_buf);

        Self {
            splat_pipeline,
            splat_bind_group,
            globals_buf,
            instance_buf,
            instance_count: instances.len() as u32,
            composite_pipeline,
            composite_bgl,
            composite_bind_group,
            comp_buf,
            accum_view,
        }
    }

    /// Recreate the size-dependent accumulator and update the viewport uniform.
    fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        self.accum_view = make_accum(device, width, height);
        self.composite_bind_group =
            make_composite_bg(device, &self.composite_bgl, &self.accum_view, &self.comp_buf);
        queue.write_buffer(
            &self.globals_buf,
            0,
            bytemuck::bytes_of(&GlobalsRaw {
                viewport: [width as f32, height as f32],
                _pad: [0.0, 0.0],
            }),
        );
    }

    /// Re-upload the per-gaussian instances (after a camera move re-runs the CPU preprocess).
    fn update_instances(&mut self, device: &wgpu::Device, instances: &[InstanceRaw]) {
        self.instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instances"),
            contents: bytemuck::cast_slice(instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        self.instance_count = instances.len() as u32;
    }

    /// Update the WSR background uniform (used when a newly loaded scene wants a different bg).
    fn update_background(&self, queue: &wgpu::Queue, bg: &Background) {
        queue.write_buffer(
            &self.comp_buf,
            0,
            bytemuck::bytes_of(&CompositeRaw {
                c_b: bg.c_b.to_array(),
                w_b: bg.w_b,
            }),
        );
    }

    fn draw(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        // Pass 1: additive splat into the accumulator (cleared to 0).
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("splat pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.accum_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.splat_pipeline);
            pass.set_bind_group(0, &self.splat_bind_group, &[]);
            pass.set_vertex_buffer(0, self.instance_buf.slice(..));
            pass.draw(0..4, 0..self.instance_count);
        }
        // Pass 2: WSR divide to the target.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("composite pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.composite_pipeline);
            pass.set_bind_group(0, &self.composite_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

fn make_accum(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("wsr accumulator"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ACCUM_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    tex.create_view(&wgpu::TextureViewDescriptor::default())
}

fn make_composite_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    accum_view: &wgpu::TextureView,
    comp_buf: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("composite bg"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(accum_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: comp_buf.as_entire_binding(),
            },
        ],
    })
}

/// Per-pane intermediate color target (linear `Rgba8Unorm`; the side-by-side blit reads these).
const PANE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Background presets cycled with the `b` key: `(w_b, c_b)`. White matches tt-splat's training
/// (`c_b`=1, `w_b`=softplus(-3)≈0.049): route-B `.ply`s carry no background, and their white
/// "empty-space" gaussians only vanish on a white background. (`w_b` affects WSR only.)
const BG_PRESETS: [(f32, [f32; 3]); 3] = [
    (0.02, [0.0, 0.0, 0.0]),    // black
    (0.0486, [1.0, 1.0, 1.0]),  // white — tt-splat route-B training background
    (0.02, [0.10, 0.10, 0.12]), // dark slate
];

fn make_color_target(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("pane color target"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: PANE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

/// Split a surface width into (left pane, right pane) widths.
fn split_widths(width: u32) -> (u32, u32) {
    let left = (width / 2).max(1);
    (left, (width - width / 2).max(1))
}

/// Which compositing model a pane renders with.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Method {
    /// Poly-splat + Weighted Sum Rendering (the tt-splat / BH method).
    Wsr,
    /// Standard 3DGS: `exp(−Q/2)`, depth-sorted alpha compositing (the original method).
    Gs,
}

/// Standard 3DGS renderer: depth-sorted instances drawn with `exp(−Q/2)` alpha and "over" blending.
struct GsRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    globals_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    instance_count: u32,
    clear: wgpu::Color,
}

impl GsRenderer {
    fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        instances: &[InstanceRaw],
        bg: &Background,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gs shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gs.wgsl").into()),
        });
        let globals_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gs globals"),
            contents: bytemuck::bytes_of(&GlobalsRaw {
                viewport: [width as f32, height as f32],
                _pad: [0.0, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gs globals bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gs globals bg"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buf.as_entire_binding(),
            }],
        });
        let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gs instances"),
            contents: bytemuck::cast_slice(instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2, 1 => Float32x2, 2 => Float32x3, 3 => Float32, 4 => Float32x3,
            ],
        };
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gs layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });
        // Painter's-order "over": rgb = src·a + dst·(1−a), a = src.a + dst.a·(1−a).
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gs pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_gs",
                buffers: &[instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_gs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        Self {
            pipeline,
            bind_group,
            globals_buf,
            instance_buf,
            instance_count: instances.len() as u32,
            clear: bg_clear(bg),
        }
    }

    fn update_instances(&mut self, device: &wgpu::Device, instances: &[InstanceRaw]) {
        self.instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gs instances"),
            contents: bytemuck::cast_slice(instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        self.instance_count = instances.len() as u32;
    }

    fn update_background(&mut self, bg: &Background) {
        self.clear = bg_clear(bg);
    }

    fn resize(&self, queue: &wgpu::Queue, width: u32, height: u32) {
        queue.write_buffer(
            &self.globals_buf,
            0,
            bytemuck::bytes_of(&GlobalsRaw {
                viewport: [width as f32, height as f32],
                _pad: [0.0, 0.0],
            }),
        );
    }

    fn draw(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("gs pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.instance_buf.slice(..));
        pass.draw(0..4, 0..self.instance_count);
    }
}

fn bg_clear(bg: &Background) -> wgpu::Color {
    wgpu::Color {
        r: bg.c_b.x as f64,
        g: bg.c_b.y as f64,
        b: bg.c_b.z as f64,
        a: 1.0,
    }
}

/// A render pane: a gaussian set, the renderer it uses, and the color target it draws into.
struct Pane {
    method: Method,
    gaussians: Vec<scene::Gaussian>,
    color_view: wgpu::TextureView,
    wsr: Option<Renderer>,
    gs: Option<GsRenderer>,
}

impl Pane {
    fn new(
        device: &wgpu::Device,
        method: Method,
        gaussians: Vec<scene::Gaussian>,
        bg: &Background,
        cam: &scene::Camera,
        width: u32,
        height: u32,
    ) -> Self {
        let color_view = make_color_target(device, width, height);
        let (wsr, gs) = match method {
            Method::Wsr => {
                let inst = scene::preprocess(&gaussians, cam, scene::WSR_SIGMAS, false);
                (Some(Renderer::new(device, PANE_FORMAT, width, height, &inst, bg)), None)
            }
            Method::Gs => {
                let inst = scene::preprocess_sorted(&gaussians, cam, scene::GS_SIGMAS, true);
                (None, Some(GsRenderer::new(device, PANE_FORMAT, width, height, &inst, bg)))
            }
        };
        Pane { method, gaussians, color_view, wsr, gs }
    }

    fn instances_for(&self, cam: &scene::Camera) -> Vec<InstanceRaw> {
        match self.method {
            Method::Wsr => scene::preprocess(&self.gaussians, cam, scene::WSR_SIGMAS, false),
            Method::Gs => scene::preprocess_sorted(&self.gaussians, cam, scene::GS_SIGMAS, true),
        }
    }

    /// Re-run the CPU preprocess for `cam` and re-upload (called on every camera move).
    fn update_camera(&mut self, device: &wgpu::Device, cam: &scene::Camera) {
        let inst = self.instances_for(cam);
        match (&mut self.wsr, &mut self.gs) {
            (Some(r), _) => r.update_instances(device, &inst),
            (_, Some(r)) => r.update_instances(device, &inst),
            _ => {}
        }
    }

    /// Replace this pane's gaussians (camera refresh + background applied separately).
    fn set_scene(&mut self, gaussians: Vec<scene::Gaussian>) {
        self.gaussians = gaussians;
    }

    /// Update just the background (WSR uses `w_b` + `c_b`; the 3DGS pane uses `c_b` as its clear).
    fn set_background(&mut self, queue: &wgpu::Queue, bg: &Background) {
        match (&self.wsr, &mut self.gs) {
            (Some(r), _) => r.update_background(queue, bg),
            (_, Some(r)) => r.update_background(bg),
            _ => {}
        }
    }

    fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cam: &scene::Camera,
        width: u32,
        height: u32,
    ) {
        self.color_view = make_color_target(device, width, height);
        match (&mut self.wsr, &self.gs) {
            (Some(r), _) => r.resize(device, queue, width, height),
            (_, Some(r)) => r.resize(queue, width, height),
            _ => {}
        }
        self.update_camera(device, cam);
    }

    fn draw(&self, encoder: &mut wgpu::CommandEncoder) {
        match (&self.wsr, &self.gs) {
            (Some(r), _) => r.draw(encoder, &self.color_view),
            (_, Some(r)) => r.draw(encoder, &self.color_view),
            _ => {}
        }
    }
}

/// `split` uniform for the blit (left pane width in pixels; 16-byte aligned).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SplitRaw {
    left_w: u32,
    _pad: [u32; 3],
}

fn make_blit_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    a: &wgpu::TextureView,
    b: &wgpu::TextureView,
    split: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("blit bg"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(a) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(b) },
            wgpu::BindGroupEntry { binding: 2, resource: split.as_entire_binding() },
        ],
    })
}

/// Window + surface + the two render panes + the shared orbit camera.
struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,

    pane_a: Pane, // left: WSR
    pane_b: Pane, // right: standard 3DGS

    blit_pipeline: wgpu::RenderPipeline,
    blit_bgl: wgpu::BindGroupLayout,
    blit_bind_group: wgpu::BindGroup,
    split_buf: wgpu::Buffer,

    orbit: scene::Orbit,
    bg_preset: usize,
    dragging: bool,
    last_cursor: Option<(f64, f64)>,
}

impl State {
    async fn new(window: Arc<Window>, gaussians: Vec<scene::Gaussian>, bg: Background) -> State {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(window.clone())
            .expect("create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("request adapter");

        let required_limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("tt-splat-viewer device"),
                    required_features: wgpu::Features::empty(),
                    required_limits,
                },
                None,
            )
            .await
            .expect("request device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Left pane = WSR (the given scene); right pane = standard 3DGS (a demo until a .ply is dropped).
        let (left_w, right_w) = split_widths(config.width);
        let h = config.height.max(1);
        let orbit = scene::Orbit::frame(&gaussians);
        let cam_a = orbit.camera(left_w, h);
        let cam_b = orbit.camera(right_w, h);
        let pane_a = Pane::new(&device, Method::Wsr, gaussians, &bg, &cam_a, left_w, h);
        let (demo_g, demo_bg) = scene::demo_scene();
        let pane_b = Pane::new(&device, Method::Gs, demo_g, &demo_bg, &cam_b, right_w, h);

        // Side-by-side blit pipeline (samples the two pane color targets into the surface halves).
        let blit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blit shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("blit.wgsl").into()),
        });
        let tex_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let blit_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blit bgl"),
            entries: &[
                tex_entry(0),
                tex_entry(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let split_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("split"),
            contents: bytemuck::bytes_of(&SplitRaw { left_w, _pad: [0; 3] }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let blit_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("blit layout"),
            bind_group_layouts: &[&blit_bgl],
            push_constant_ranges: &[],
        });
        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blit pipeline"),
            layout: Some(&blit_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: "vs_blit",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: "fs_blit",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let blit_bind_group =
            make_blit_bg(&device, &blit_bgl, &pane_a.color_view, &pane_b.color_view, &split_buf);

        State {
            surface,
            device,
            queue,
            config,
            size,
            window,
            pane_a,
            pane_b,
            blit_pipeline,
            blit_bgl,
            blit_bind_group,
            split_buf,
            orbit,
            bg_preset: 0,
            dragging: false,
            last_cursor: None,
        }
    }

    fn current_bg(&self) -> Background {
        let (w_b, c_b) = BG_PRESETS[self.bg_preset];
        Background { w_b, c_b: glam::Vec3::from_array(c_b) }
    }

    /// Replace one pane's scene (e.g. a dropped `.ply`): re-frame the shared orbit, refresh both panes.
    fn set_scene_pane(&mut self, pane: usize, gaussians: Vec<scene::Gaussian>) {
        self.orbit = scene::Orbit::frame(&gaussians);
        let pane = if pane == 0 { &mut self.pane_a } else { &mut self.pane_b };
        pane.set_scene(gaussians);
        self.refresh_view();
    }

    /// Cycle the shared background preset (the `b` key); applied to both panes.
    fn cycle_background(&mut self) {
        self.bg_preset = (self.bg_preset + 1) % BG_PRESETS.len();
        let bg = self.current_bg();
        self.pane_a.set_background(&self.queue, &bg);
        self.pane_b.set_background(&self.queue, &bg);
        self.window.request_redraw();
    }

    /// Re-run the CPU preprocess for both panes at the current shared camera.
    fn refresh_view(&mut self) {
        let (left_w, right_w) = split_widths(self.config.width);
        let h = self.config.height.max(1);
        let cam_a = self.orbit.camera(left_w, h);
        let cam_b = self.orbit.camera(right_w, h);
        self.pane_a.update_camera(&self.device, &cam_a);
        self.pane_b.update_camera(&self.device, &cam_b);
        self.window.request_redraw();
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        let (left_w, right_w) = split_widths(new_size.width);
        let h = new_size.height;
        let cam_a = self.orbit.camera(left_w, h);
        let cam_b = self.orbit.camera(right_w, h);
        self.pane_a.resize(&self.device, &self.queue, &cam_a, left_w, h);
        self.pane_b.resize(&self.device, &self.queue, &cam_b, right_w, h);
        self.queue.write_buffer(
            &self.split_buf,
            0,
            bytemuck::bytes_of(&SplitRaw { left_w, _pad: [0; 3] }),
        );
        self.blit_bind_group = make_blit_bg(
            &self.device,
            &self.blit_bgl,
            &self.pane_a.color_view,
            &self.pane_b.color_view,
            &self.split_buf,
        );
        self.window.request_redraw();
    }

    /// Handle a mouse-drag delta (orbit) — yaw/pitch around the target.
    fn orbit_drag(&mut self, dx: f64, dy: f64) {
        let sens = 0.005;
        self.orbit.yaw -= dx as f32 * sens;
        self.orbit.pitch = (self.orbit.pitch + dy as f32 * sens)
            .clamp(-1.5, 1.5); // keep just shy of the poles (±~86°)
        self.refresh_view();
    }

    /// Handle a scroll delta (dolly in/out).
    fn zoom(&mut self, scroll: f32) {
        self.orbit.radius = (self.orbit.radius * (1.0 - scroll * 0.1)).max(1e-2);
        self.refresh_view();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });
        // Each pane renders into its own color target, then the blit pass places them side by side.
        self.pane_a.draw(&mut encoder);
        self.pane_b.draw(&mut encoder);
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.blit_pipeline);
            pass.set_bind_group(0, &self.blit_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }

    /// Which pane a cursor x-coordinate (physical px) is over: 0 = left (WSR), 1 = right (3DGS).
    #[cfg(not(target_arch = "wasm32"))]
    fn pane_at_x(&self, x: f64) -> usize {
        let (left_w, _) = split_widths(self.config.width);
        if x < left_w as f64 {
            0
        } else {
            1
        }
    }
}

/// WASM entry: `wasm_bindgen(start)` must be sync, so kick the async `run()` onto the microtask queue.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_start() {
    wasm_bindgen_futures::spawn_local(run());
}

// Actions from JS (the web has no winit file-drop / reliable canvas key focus), applied by the event
// loop on the next tick: a dropped scene (pane + gaussians) and a background-cycle request.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static PENDING_SCENE: std::cell::RefCell<Option<(usize, Vec<scene::Gaussian>)>> =
        const { std::cell::RefCell::new(None) };
    static PENDING_BG_CYCLE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Called from JS drag-and-drop with the target pane (0 = left/WSR, 1 = right/3DGS) and file bytes.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn load_ply_into(pane: u32, bytes: &[u8]) -> bool {
    match scene::parse_ply(bytes) {
        Ok(g) if !g.is_empty() => {
            log::info!("dropped .ply into pane {pane}: {} gaussians", g.len());
            let pane = (pane as usize).min(1);
            PENDING_SCENE.with(|p| *p.borrow_mut() = Some((pane, g)));
            true
        }
        Ok(_) => {
            log::error!("dropped .ply has no gaussians");
            false
        }
        Err(e) => {
            log::error!("failed to parse dropped .ply: {e}");
            false
        }
    }
}

/// Called from JS (the `b` key) to cycle the background preset.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn cycle_background() {
    PENDING_BG_CYCLE.with(|c| c.set(true));
}

/// Entry point shared by native (`main.rs`) and WASM (`wasm_start`).
pub async fn run() {
    init_logging();

    let event_loop = EventLoop::new().expect("create event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("tt-splat-viewer")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
            .build(&event_loop)
            .expect("create window"),
    );

    #[cfg(target_arch = "wasm32")]
    attach_canvas(&window);

    let (gaussians, bg) = load_scene();
    log::info!("loaded {} gaussians", gaussians.len());
    let mut state = State::new(window.clone(), gaussians, bg).await;
    state.window.request_redraw();

    let handler = move |event: Event<()>, elwt: &winit::event_loop::EventLoopWindowTarget<()>| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Wait);
        // Keep redrawing so a late surface-size (common on web) and orbit moves are always shown.
        if let Event::AboutToWait = event {
            // Apply actions handed in from JS (web has no winit file-drop / reliable key focus).
            #[cfg(target_arch = "wasm32")]
            {
                if let Some((pane, g)) = PENDING_SCENE.with(|p| p.borrow_mut().take()) {
                    state.set_scene_pane(pane, g);
                }
                if PENDING_BG_CYCLE.with(|c| c.replace(false)) {
                    state.cycle_background();
                }
            }
            state.window.request_redraw();
            return;
        }
        if let Event::WindowEvent { event, window_id } = event {
            if window_id != state.window.id() {
                return;
            }
            match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(new_size) => state.resize(new_size),
                #[cfg(not(target_arch = "wasm32"))]
                WindowEvent::DroppedFile(path) => {
                    if path.extension().and_then(|e| e.to_str()) == Some("ply") {
                        // Route to the pane under the cursor (left = WSR, right = 3DGS).
                        let pane = state.last_cursor.map_or(0, |(x, _)| state.pane_at_x(x));
                        match scene::load_ply(&path) {
                            Ok(g) if !g.is_empty() => {
                                log::info!("dropped {} → pane {pane}: {} gaussians", path.display(), g.len());
                                state.set_scene_pane(pane, g);
                            }
                            Ok(_) => log::warn!("dropped .ply has no gaussians"),
                            Err(e) => log::error!("failed to load {}: {e}", path.display()),
                        }
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                    if key_event.state == winit::event::ElementState::Pressed
                        && matches!(key_event.logical_key.as_ref(), winit::keyboard::Key::Character("b"))
                    {
                        state.cycle_background();
                    }
                }
                WindowEvent::MouseInput { state: btn, button, .. } => {
                    if button == winit::event::MouseButton::Left {
                        state.dragging = btn == winit::event::ElementState::Pressed;
                        if !state.dragging {
                            state.last_cursor = None;
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let p = (position.x, position.y);
                    if state.dragging {
                        if let Some((lx, ly)) = state.last_cursor {
                            state.orbit_drag(p.0 - lx, p.1 - ly);
                        }
                    }
                    state.last_cursor = Some(p);
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let scroll = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                        winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32 / 60.0,
                    };
                    state.zoom(scroll);
                }
                WindowEvent::RedrawRequested => match state.render() {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => log::warn!("surface error: {e:?}"),
                },
                _ => {}
            }
        }
    };

    // Native blocks in `run`; web can't block, so hand the loop to the browser via `spawn`.
    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run(handler).expect("run event loop");
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn(handler);
    }
}

/// Pick the scene: a `.ply` path given on the command line (native), else the synthetic scene.
fn load_scene() -> (Vec<scene::Gaussian>, Background) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = std::env::args().nth(1) {
            if path.ends_with(".ply") {
                match scene::load_ply(std::path::Path::new(&path)) {
                    Ok(g) => {
                        // Standard .ply has no WSR background; use a dim near-zero default.
                        let bg = Background { w_b: 0.02, c_b: glam::Vec3::ZERO };
                        return (g, bg);
                    }
                    Err(e) => log::error!("failed to load {path}: {e} — falling back to demo scene"),
                }
            }
        }
    }
    scene::demo_scene()
}

fn init_logging() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_error_panic_hook::set_once();
            let _ = console_log::init_with_level(log::Level::Info);
        } else {
            let _ = env_logger::try_init();
        }
    }
}

/// On the web, winit creates a detached canvas — size it and append it to the page so it is visible.
#[cfg(target_arch = "wasm32")]
fn attach_canvas(window: &Window) {
    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let body = doc.body()?;
            let canvas = window.canvas()?; // web_sys::HtmlCanvasElement
            canvas.set_width(1280);
            canvas.set_height(720);
            body.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("append canvas to document body");
}
