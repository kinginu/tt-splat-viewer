//! Headless render → RGBA8 pixels, for the PSNR validation harness (CLAUDE.md §5).
//!
//! No window/surface: render the WSR two-pass pipeline into an off-screen `Rgba8Unorm` target
//! (linear, NOT sRGB — so the stored bytes are `round(C·255)`, matching the oracle's
//! `(img.clamp(0,1)*255)`), copy it back, and return row-major RGBA8.

use wgpu::util::DeviceExt;

use crate::{
    make_blit_bg, scene::Background, scene::Camera, scene::Gaussian, scene::InstanceRaw,
    split_widths, GsRenderer, Method, Pane, Renderer, SplitRaw,
};

const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn align_up(value: u32, align: u32) -> u32 {
    ((value + align - 1) / align) * align
}

/// Render `instances` (already preprocessed for `cam`) headlessly. Returns `(width, height, rgba8)`.
/// `gs = true` uses the standard-3DGS renderer (instances must be depth-sorted), else WSR.
pub async fn render(cam: &Camera, instances: &[InstanceRaw], bg: &Background, gs: bool) -> (u32, u32, Vec<u8>) {
    let (width, height) = (cam.width, cam.height);

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .expect("request adapter (offscreen)");
    log::info!("offscreen adapter: {:?}", adapter.get_info().name);
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("offscreen device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("request device (offscreen)");

    // One of the two renderers (boxed as a closure to keep the readback path shared).
    let wsr = (!gs).then(|| Renderer::new(&device, TARGET_FORMAT, width, height, instances, bg));
    let gsr = gs.then(|| GsRenderer::new(&device, TARGET_FORMAT, width, height, instances, bg));

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TARGET_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let bytes_per_pixel = 4u32;
    let padded_bpr = align_up(width * bytes_per_pixel, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (padded_bpr * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("offscreen encoder"),
    });
    match (&wsr, &gsr) {
        (Some(r), _) => r.draw(&mut encoder, &target_view),
        (_, Some(r)) => r.draw(&mut encoder, &target_view),
        _ => unreachable!(),
    }
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &readback,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));

    // Map and wait.
    let (tx, rx) = std::sync::mpsc::channel();
    readback.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv().expect("map channel").expect("map readback");

    let mapped = readback.slice(..).get_mapped_range();
    let row_bytes = (width * bytes_per_pixel) as usize;
    let mut out = Vec::with_capacity(row_bytes * height as usize);
    for row in 0..height as usize {
        let start = row * padded_bpr as usize;
        out.extend_from_slice(&mapped[start..start + row_bytes]);
    }
    drop(mapped);
    readback.unmap();

    (width, height, out)
}

/// Headless render of the full dual-pane path (two `Pane`s + the side-by-side blit), mirroring the
/// windowed `State::render`. Lets the dual view be verified on the dev box without a browser.
pub async fn render_dual(
    left: &[Gaussian],
    right: &[Gaussian],
    bg: &Background,
    width: u32,
    height: u32,
) -> (u32, u32, Vec<u8>) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .expect("request adapter (dual)");
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .expect("request device (dual)");

    let (left_w, right_w) = split_widths(width);
    let orbit = crate::scene::Orbit::frame(left);
    let cam_a = orbit.camera(left_w, height);
    let cam_b = orbit.camera(right_w, height);
    let pane_a = Pane::new(&device, Method::Wsr, left.to_vec(), bg, &cam_a, left_w, height);
    let pane_b = Pane::new(&device, Method::Gs, right.to_vec(), bg, &cam_b, right_w, height);

    // Blit pipeline (same as State::new, target = TARGET_FORMAT for readback).
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("blit shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("blit.wgsl").into()),
    });
    let tex = |b| wgpu::BindGroupLayoutEntry {
        binding: b,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("blit bgl"),
        entries: &[
            tex(0),
            tex(1),
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
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("blit pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_blit",
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_blit",
            targets: &[Some(wgpu::ColorTargetState {
                format: TARGET_FORMAT,
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
    let bind = make_blit_bg(&device, &bgl, &pane_a.color_view, &pane_b.color_view, &split_buf);

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("dual target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TARGET_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let padded_bpr = align_up(width * 4, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (padded_bpr * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    pane_a.draw(&mut encoder);
    pane_b.draw(&mut encoder);
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
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &readback,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(encoder.finish()));

    let (tx, rx) = std::sync::mpsc::channel();
    readback.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv().expect("map channel").expect("map readback");
    let mapped = readback.slice(..).get_mapped_range();
    let row_bytes = (width * 4) as usize;
    let mut out = Vec::with_capacity(row_bytes * height as usize);
    for row in 0..height as usize {
        let start = row * padded_bpr as usize;
        out.extend_from_slice(&mapped[start..start + row_bytes]);
    }
    drop(mapped);
    readback.unmap();
    (width, height, out)
}
