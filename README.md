# tt-splat-viewer

A **Rust + wgpu** interactive viewer for the Gaussian-splatting rendering model explored in the
sibling research project [tt-splat](https://github.com/kinginu/tt-splat) — poly-splat + Weighted Sum
Rendering, which is order-independent (no depth sort, no transcendentals).

Native (desktop) + WASM (browser).

> **Scope:** this is a *reference / visual de-risk / demo* tool running on a normal GPU via wgpu.
> It is **not** a performance artifact — the parent project's perf thesis targets Tenstorrent
> Blackhole silicon, which this viewer does not touch. No perf claims come from here. Its job is to
> *see* the tt-splat model on real scenes and de-risk its known weaknesses (depth-free occlusion; scale).

## Status

Early. The WSR renderer (CPU projection → additive poly-splat quads → fullscreen divide) is
implemented and **numerically validated against the PyTorch reference** (offscreen render diffed by
PSNR; see `validation/`). Next: `.ply` loading and interactive camera controls.

## License

MIT (see [`LICENSE`](./LICENSE)). Some wgpu/WASM/`.ply`/camera plumbing is adapted from
[abist-co-ltd/wgpu-gs-viewer](https://github.com/abist-co-ltd/wgpu-gs-viewer) (MIT,
© 2026 株式会社アビスト イノベーションセンター); adapted files keep their original copyright header.
