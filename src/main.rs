//! Native entry point. The browser build uses the `#[wasm_bindgen(start)]` `run()` in `lib.rs`.

fn main() {
    pollster::block_on(tt_splat_viewer::run());
}
