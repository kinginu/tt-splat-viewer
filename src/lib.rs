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

use scene::{Background, Camera, InstanceRaw};

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

/// Window + surface + the WSR renderer.
struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,
    renderer: Renderer,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
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

        // Build the synthetic scene and preprocess it for this camera.
        let (gaussians, bg) = scene::synthetic_scene();
        let cam = Camera::look_at(
            glam::Vec3::new(0.0, 0.0, 0.0),
            glam::Vec3::new(0.0, 0.0, 1.0),
            glam::Vec3::new(0.0, 1.0, 0.0),
            config.width as f32, // fx ≈ width (≈53° hfov) — placeholder until real intrinsics
            config.width as f32,
            config.width,
            config.height,
        );
        let instances = scene::preprocess(&gaussians, &cam);
        let renderer = Renderer::new(&device, format, config.width, config.height, &instances, &bg);

        State {
            surface,
            device,
            queue,
            config,
            size,
            window,
            renderer,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.renderer
                .resize(&self.device, &self.queue, new_size.width, new_size.height);
        }
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
        self.renderer.draw(&mut encoder, &view);
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }
}

/// Entry point shared by native (`main.rs`) and WASM (`#[wasm_bindgen(start)]`).
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    init_logging();

    let event_loop = EventLoop::new().expect("create event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("tt-splat-viewer")
            .build(&event_loop)
            .expect("create window"),
    );

    #[cfg(target_arch = "wasm32")]
    attach_canvas(&window);

    let mut state = State::new(window.clone()).await;

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(winit::event_loop::ControlFlow::Wait);
            if let Event::WindowEvent { event, window_id } = event {
                if window_id != state.window.id() {
                    return;
                }
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(new_size) => state.resize(new_size),
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
            } else if let Event::AboutToWait = event {
                state.window.request_redraw();
            }
        })
        .expect("run event loop");
}

fn init_logging() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_error_panic_hook::set_once();
            let _ = console_log::init_with_level(log::Level::Warn);
        } else {
            let _ = env_logger::try_init();
        }
    }
}

/// On the web, winit creates a detached canvas — append it to the page so it is visible.
#[cfg(target_arch = "wasm32")]
fn attach_canvas(window: &Window) {
    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let body = doc.body()?;
            let canvas = web_sys::Element::from(window.canvas()?);
            body.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("append canvas to document body");
}
