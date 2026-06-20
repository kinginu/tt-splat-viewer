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
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // Optional leading --gs flag renders with the standard-3DGS pipeline instead of WSR.
    let gs = args.first().map(String::as_str) == Some("--gs");
    if gs {
        args.remove(0);
    }
    let mut args = args.into_iter();
    let input = args.next().expect("usage: offscreen [--gs] <scene.json|model.ply|--demo|--dual> <out.png> [view.json | yaw pitch]");
    let out_path = args.next().expect("usage: offscreen [--gs] <scene.json|model.ply|--demo|--dual> <out.png> [view.json | yaw pitch]");

    // --dual: render the full two-pane path (WSR | 3DGS) of the default sample, like the window does.
    if input == "--dual" {
        let (g, bg) = scene::synthetic_scene();
        let (w, h, rgba) = pollster::block_on(offscreen::render_dual(&g, &g, &bg, 1280, 720));
        image::RgbaImage::from_raw(w, h, rgba).expect("image").save(&out_path).expect("save");
        eprintln!("wrote {out_path} (dual)");
        return;
    }

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

    let instances = if gs {
        scene::preprocess_sorted(&gaussians, &cam, scene::GS_SIGMAS, true)
    } else {
        scene::preprocess(&gaussians, &cam, scene::WSR_SIGMAS, false)
    };
    eprintln!(
        "rendering {} gaussians ({} kept) at {}x{} [{}]",
        gaussians.len(),
        instances.len(),
        cam.width,
        cam.height,
        if gs { "3DGS" } else { "WSR" }
    );

    let (w, h, rgba) = pollster::block_on(offscreen::render(&cam, &instances, &bg, gs));
    image::RgbaImage::from_raw(w, h, rgba)
        .expect("build image")
        .save(&out_path)
        .expect("save png");
    eprintln!("wrote {out_path}");
}
