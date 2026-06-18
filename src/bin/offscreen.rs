//! Headless render of a `scene.json` to a PNG — the Rust side of the PSNR harness (CLAUDE.md §5).
//!
//!   cargo run --bin offscreen -- validation/scene.json validation/rust.png

use std::path::Path;

use tt_splat_viewer::{offscreen, scene};

fn main() {
    let _ = env_logger::try_init();
    let mut args = std::env::args().skip(1);
    let scene_path = args.next().expect("usage: offscreen <scene.json> <out.png>");
    let out_path = args.next().expect("usage: offscreen <scene.json> <out.png>");

    let (gaussians, bg, cam) =
        scene::load_scene_json(Path::new(&scene_path)).expect("load scene.json");
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
