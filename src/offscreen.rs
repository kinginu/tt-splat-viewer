//! Headless render → RGBA8 pixels, for the PSNR validation harness (CLAUDE.md §5).
//!
//! No window/surface: render the WSR two-pass pipeline into an off-screen `Rgba8Unorm` target
//! (linear, NOT sRGB — so the stored bytes are `round(C·255)`, matching the oracle's
//! `(img.clamp(0,1)*255)`), copy it back, and return row-major RGBA8.

use crate::{scene::Background, scene::Camera, scene::InstanceRaw, Renderer};

const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

fn align_up(value: u32, align: u32) -> u32 {
    ((value + align - 1) / align) * align
}

/// Render `instances` (already preprocessed for `cam`) headlessly. Returns `(width, height, rgba8)`.
pub async fn render(cam: &Camera, instances: &[InstanceRaw], bg: &Background) -> (u32, u32, Vec<u8>) {
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

    let renderer = Renderer::new(&device, TARGET_FORMAT, width, height, instances, bg);

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
    renderer.draw(&mut encoder, &target_view);
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
