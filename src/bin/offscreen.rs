//! Headless render to a PNG — the Rust side of the PSNR harness (CLAUDE.md §5).
//!
//!   # full scene (gaussians + camera + bg in one file):
//!   cargo run --bin offscreen -- validation/scene.json out.png
//!   # a .ply with an explicit oracle-matched camera/bg (PSNR harness):
//!   cargo run --bin offscreen -- model.ply out.png view.json
//!   # a .ply auto-framed by the orbit camera, optional yaw/pitch in degrees (exercises Orbit):
//!   cargo run --bin offscreen -- model.ply out.png --orbit 30 15

use std::path::Path;

use tt_splat_viewer::{offscreen, scene};

fn main() {
    let _ = env_logger::try_init();
    let mut args = std::env::args().skip(1);
    let input = args.next().expect("usage: offscreen <scene.json|model.ply> <out.png> [view.json | --orbit yaw pitch]");
    let out_path = args.next().expect("usage: offscreen <scene.json|model.ply> <out.png> [view.json | --orbit yaw pitch]");
    let third = args.next();

    let (gaussians, bg, cam) = if input == "--demo" {
        // Procedural demo scene (same as the WASM/native default), auto-framed; optional yaw/pitch.
        let (g, bg) = scene::demo_scene();
        let mut orbit = scene::Orbit::frame(&g);
        if let Some(y) = third {
            orbit.yaw = y.parse::<f32>().unwrap_or(0.0).to_radians();
        }
        if let Some(p) = args.next() {
            orbit.pitch = p.parse::<f32>().unwrap_or(0.0).to_radians();
        }
        let cam = orbit.camera(800, 800);
        (g, bg, cam)
    } else if input.ends_with(".ply") {
        let g = scene::load_ply(Path::new(&input)).expect("load .ply");
        match third.as_deref() {
            Some("--orbit") | None => {
                // Auto-frame via the same Orbit the interactive window uses.
                let (w, h) = (800u32, 800u32);
                let mut orbit = scene::Orbit::frame(&g);
                if let Some(y) = args.next() {
                    orbit.yaw = y.parse::<f32>().unwrap_or(0.0).to_radians();
                }
                if let Some(p) = args.next() {
                    orbit.pitch = p.parse::<f32>().unwrap_or(0.0).to_radians();
                }
                let bg = scene::Background { w_b: 0.02, c_b: glam::Vec3::ZERO };
                (g, bg, orbit.camera(w, h))
            }
            Some(view) => {
                let (bg, cam) = scene::load_view_json(Path::new(view)).expect("load view.json");
                (g, bg, cam)
            }
        }
    } else {
        scene::load_scene_json(Path::new(&input)).expect("load scene.json")
    };

    let instances = scene::preprocess(&gaussians, &cam);
    eprintln!(
        "rendering {} gaussians ({} kept) at {}x{}",
        gaussians.len(),
        instances.len(),
        cam.width,
        cam.height
    );

    let (w, h, rgba) = pollster::block_on(offscreen::render(&cam, &instances, &bg));
    image::RgbaImage::from_raw(w, h, rgba)
        .expect("build image")
        .save(&out_path)
        .expect("save png");
    eprintln!("wrote {out_path}");
}
