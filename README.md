# tt-splat-viewer

A **Rust + wgpu** interactive viewer for the **"(B) route"** Gaussian-splatting renderer
(poly-splat + Weighted Sum Rendering — order-independent, no depth sort, no transcendentals),
the rendering model explored in the sibling research project [tt-splat](https://github.com/kinginu/tt-splat).

Native (desktop) + WASM (browser).

> **Scope:** this is a *reference / visual de-risk / demo* tool running on a normal GPU via wgpu.
> It is **not** a performance artifact — the parent project's perf thesis targets Tenstorrent
> Blackhole silicon, which this viewer does not touch. No perf claims come from here. Its job is to
> *see* the (B) model on real scenes and de-risk its known weaknesses (depth-free occlusion; scale).

## Status

Bootstrapping. **Start here:** [`HANDOFF.md`](./HANDOFF.md) — the full design + math + build plan.

## License

MIT (see [`LICENSE`](./LICENSE)). Some wgpu/WASM/`.ply`/camera plumbing is adapted from
[abist-co-ltd/wgpu-gs-viewer](https://github.com/abist-co-ltd/wgpu-gs-viewer) (MIT,
© 2026 株式会社アビスト イノベーションセンター); adapted files keep their original copyright header.
