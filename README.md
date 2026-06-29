# tt-splat-viewer

A **Rust + wgpu** interactive viewer for the Gaussian-splatting rendering model explored in the
sibling research project [tt-splat](https://github.com/kinginu/tt-splat) — poly-splat + Weighted Sum
Rendering, which is order-independent (no depth sort, no transcendentals).

Native (desktop) + WASM (browser).

![demo: a gaussian set loaded from a .ply, orbited](docs/demo.png)

> **Scope:** this is a *reference / visual de-risk / demo* tool running on a normal GPU via wgpu.
> It is **not** a performance artifact — the parent project's perf thesis targets Tenstorrent
> Blackhole silicon, which this viewer does not touch. No perf claims come from here. Its job is to
> *see* the tt-splat model on real scenes and de-risk its known weaknesses (depth-free occlusion; scale).

## Live demo (no install)

**→ https://kinginu.github.io/tt-splat-viewer/** — open in a WebGPU browser (Chrome/Edge, or
Safari 17.4+) and **drop a `.ply` onto a pane**. Left = WSR (the faithful render), right = standard
3DGS (for comparison). Left-drag orbits, right-drag pans, scroll zooms, `b` cycles the background.

**Why a dedicated viewer:** tt-splat exports its trained model as a standard-layout `.ply`, but those
gaussians are **fit to the WSR renderer** (poly-splat `(1−Q/k)₊²` + Weighted Sum Rendering — no `exp`,
no depth sort). A standard 3DGS viewer renders with `exp`-splat + sorted alpha and will **not**
reproduce the image. This viewer renders WSR faithfully (and shows standard 3DGS side-by-side).
Route-B models also train against a **white** background (`b` → white) since the `.ply` carries no
background term.

## Usage

```sh
# Interactive viewer (native; needs a display). Drag to orbit, scroll to zoom.
cargo run --release -- path/to/model.ply      # omit the arg for a built-in synthetic scene

# Headless render to a PNG (no display needed):
cargo run --bin offscreen -- model.ply out.png --orbit 45 20   # auto-framed, yaw/pitch in degrees

# Browser (WebGPU): build the wasm bundle locally, then static-serve web/.
cargo build --lib --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/tt_splat_viewer.wasm --out-dir web --target web --no-typescript
cd web && python3 -m http.server 8000   # open http://localhost:8000 (localhost is a WebGPU secure context)
```

The public live demo above is built and deployed to GitHub Pages by CI
(`.github/workflows/pages.yml`) on every push to `main` — the wasm/js are generated, not committed.

A standard INRIA-3DGS `.ply` is rendered through the WSR model directly — see CLAUDE.md §4 on what
that does and does not show.

## Status

The WSR renderer (CPU projection → additive poly-splat quads → fullscreen divide), a standard-3DGS
`.ply` loader, and an orbit camera are implemented and **numerically validated against the PyTorch
reference** — offscreen renders diffed by PSNR (3-gaussian: 50 dB; 2000-gaussian `.ply`: 67 dB; both
within 8-bit rounding). See `validation/`. Next: real captured scenes, faithful tt-splat exports, and
the scale/occlusion explorations.

## License

MIT (see [`LICENSE`](./LICENSE)). Plumbing here (wgpu/WASM init, `.ply` parsing, camera) is currently
original. Per the parent design notes it may later borrow isolated pieces from
[abist-co-ltd/wgpu-gs-viewer](https://github.com/abist-co-ltd/wgpu-gs-viewer) (MIT,
© 2026 株式会社アビスト イノベーションセンター); any such adapted file keeps its original copyright header.
