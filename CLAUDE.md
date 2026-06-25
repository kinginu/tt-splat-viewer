# tt-splat-viewer — project context for Claude Code

Working context for this repo. Read top to bottom before writing code. The public-facing
`README.md` is deliberately terse; **this file is the real design + math + build plan.**

## Repo / git policy

- Only **source code and this `CLAUDE.md`** are tracked in git. No other docs, notes, or handoff
  files get committed — keep internal design notes here in `CLAUDE.md`, not in separate `.md` files.
- This repo: `github.com/kinginu/tt-splat-viewer` (private), MIT.
- Parent reference (readable, same machine): `../tt-splat` (private, `github.com/kinginu/tt-splat`).
  Start with `spike/{render,arms,forward,geometry,camera}.py`.
- Git identity is already set (`kinginu`). Commit/push to `main` when the user asks.

---

## 0. What you are building (TL;DR)

A **from-scratch Rust + wgpu + WGSL** interactive viewer for the tt-splat Gaussian-splatting
renderer: **poly-splat + Weighted Sum Rendering (WSR)** — order-independent, **no depth sort, no
transcendentals**. Native (desktop) + WASM (browser).

**Purpose (be honest about scope):** this is a *reference / visual de-risk / demo* tool. It runs on a
normal GPU via wgpu. **It is NOT the performance artifact** — the parent project's perf thesis is about
running on Tenstorrent Blackhole silicon, which this viewer does not touch. **No perf/$ claim ever comes
from this repo.** Its job is to let us *see* the tt-splat model on real scenes and de-risk its known
weaknesses (below).

---

## 1. Parent project context (`tt-splat`)

- tt-splat maps 3D Gaussian Splatting onto **Tenstorrent Blackhole**. The renderer this viewer targets
  swaps standard 3DGS's sorted-alpha compositing for a hardware-friendly formulation:
  - **No depth sort** → Weighted Sum Rendering (WSR), order-independent.
  - **No transcendentals** → poly-splat weight `(1−Q/k)₊²` instead of `exp(−Q/2)`.
  - **Matmul-shaped** → the quadratic form `Q = Φ·θᵀ` is an exact GEMM.
- Phase 1 (hardware-independent) is **complete**: the math is de-risked (depth-free WSR ≈ parity
  with standard 3DGS on the ficus scene), forward+backward validated on the TT simulator (ttsim), and
  fixed-K binning shown lossless. Everything past that needs real silicon.

**Why a viewer exists — the weaknesses to de-risk visually:**
- **A1 (occlusion):** WSR is depth-free → it *averages* overlapping gaussians and **cannot model
  occlusion** the way sorted-alpha does. We only validated ≈parity on ONE small synthetic object
  (ficus). This viewer should let us *see* where WSR breaks on occluded/depth-complex scenes.
- **A3 (scale):** fixed-K binning was only tested at G=2000 / res 128. Behavior at higher gaussian
  counts and resolutions is untested. The viewer is where we explore that interactively.

---

## 2. The rendering math — match the PyTorch oracle EXACTLY

The ground truth is the PyTorch reference in **`../tt-splat/spike/`**. Do not trust the formulas below
over the code — **mirror the code, and verify numerically (§5)**. Key files:
- `spike/geometry.py` — `quat_to_rotmat`, `cov3d`, `project_ewa` (world → 2D mean + conic + depth + keep).
- `spike/forward.py` — `quad_form` (Q), `poly_splat_wgeo` (w_geo), `color_from_dc`.
- `spike/arms.py` — **`blend_A`** ← THE renderer to reproduce.
- `spike/render.py` — the full dense pipeline (geometry → Q → w_geo → blend_A).
- `spike/camera.py` — camera conventions + `pixel_grid` (pixel centers at `col+0.5, row+0.5`).

**Per frame**, given gaussians `{mean3d, scale, quat, color_dc, opacity_raw}`, learnable background
`{w_b, c_b}`, and a camera:

**Geometry (CPU/preprocess is fine — it is per-gaussian setup, not the hot loop):**
project each gaussian to a 2D mean `mu2d[g]`, a 2D conic `(a,b,c)[g]` (entries of the inverse 2D
covariance), a `depth`, and a `keep` visibility flag (EWA projection — see `project_ewa`).

**Per-pixel hot path (your WGSL):** for pixel `P` and gaussian `g`, with `(dx,dy) = P − mu2d[g]`:
```
Q      = a·dx² + 2·b·dx·dy + c·dy²          # quadratic form (match forward.quad_form)
w_geo  = max(0, 1 − Q/k)²                    # poly-splat, k = 4.0 (constant). Zero beyond Q ≥ k.
o      = sigmoid(opacity_raw[g])
w      = o · w_geo
```
**WSR blend (order-independent — this is the whole point):**
```
N = Σ_g w · color[g]   +  w_b · c_b          # numerator  (RGB)
D = Σ_g w              +  w_b                 # denominator
C = N / D                                     # final pixel color
```
(`blend_A` = `_wsr(o·w_geo, color, w_b, c_b)` with `num = W@color + w_b·c_b`, `den = W.sum + w_b`,
`C = num/den`.) Color is `color_from_dc`: `clamp(0.5 + C0·dc, min=0)`, `C0 = 0.28209479...` (SH deg-0).

**Why this is EASY in wgpu (and why we do NOT fork a sorted-3DGS viewer):** N and D are commutative
sums, so render every gaussian's footprint with **additive blending** into an `Rgba16Float` target
(`RGB = Σ w·color`, `A = Σ w`), then one fullscreen pass does `C = (RGB + w_b·c_b)/(A + w_b)`.
**No sort, no depth test, no tile-range compaction.** The hardest parts of a normal GS viewer simply
don't exist here.

**bf16 / coordinate note:** the TT kernels use tile-local pixel coords (0–15) so `Q` stays small in
bf16. In WGSL you have f32, so global coords are fine — but keep the tile-local trick in mind if you
later add tiling.

---

## 3. Architecture decisions (already made — do not relitigate)

1. **Stack = Rust + wgpu + WGSL**, native + WASM. Chosen so we can render **offscreen natively → PNG**
   and auto-diff against the PyTorch oracle (§5), matching tt-splat's "validate against the oracle
   headlessly" discipline. Browser is the *final* visual step, not the validation loop.
2. **Renderer written from scratch.** WSR's additive blend is simple; a sorted-3DGS codebase is built
   around the depth sort we delete. Do not inherit that architecture.
3. **Lift only isolated plumbing** (with attribution) from **abist-co-ltd/wgpu-gs-viewer** (MIT,
   tag `v-0.1.0`, https://github.com/abist-co-ltd/wgpu-gs-viewer): wgpu/WASM init, `.ply` parsing,
   camera/orbit-fly controls. **Any file you copy/adapt MUST keep its MIT copyright header**
   (`© 2026 株式会社アビスト イノベーションセンター`). See `LICENSE`.
4. **Minimal first:** no tiling, no compute pass, no culling — just project → instanced quads with the
   poly+WSR shader + additive blend → fullscreen divide. Add tiling / bounding-radius cull / fixed-K
   binning later, only for the A3 scale tests.

---

## 4. The data gotcha (read before loading any `.ply`)

Standard 3DGS `.ply` files are trained for **sorted-alpha + spherical harmonics + exp** — NOT our model.
Two distinct modes:

- **(a) Quick A1 qualitative test (no data work):** load a *standard* pretrained `.ply` and render it
  with the WSR shader anyway. The result will look "wrong" — WSR averages through occlusion — and
  **that is the point**: it makes the depth-free occlusion failure visible. Fastest path to a picture.
  (Grab any pretrained 3DGS `.ply`, e.g. the abist repo's `scenes/luigi.ply`.)
- **(b) Faithful viewer (the real artifact):** consume **our** trained gaussians exported from
  tt-splat — `mean3d`, our conic/θ, `color_dc`, `opacity_raw`, `w_b`, `c_b`. This needs a small
  exporter on the tt-splat side and a matching loader here. This is what actually shows the tt-splat model.

---

## 5. Validation discipline (keep it — this is how tt-splat works)

Do **not** eyeball-only. The loop:
1. Render a **fixed camera + fixed gaussians** offscreen (native wgpu → texture → buffer → PNG via the
   `image` crate).
2. Render the *same* camera + gaussians in PyTorch using `../tt-splat/spike/render.py` (arm `A`).
3. Compute **PSNR** between the two images. WGSL is f32, the reference is f32/f64 — expect a near-exact
   match (**target > 50 dB**); a low PSNR means a convention bug (conic factor, pixel-center offset,
   premultiply, background term).

Pattern to copy: `../tt-splat/tools/m2_forward_ttsim.py` diffs a *simulator* render against
`arms.blend_A`. Same idea, different backend — you are doing it for the wgpu render.

---

## 6. Milestones (commit at each)

1. Skeleton: `cargo run` opens a wgpu window that clears the screen; `wasm-pack`/trunk path builds for
   browser. (Lift wgpu/WASM init from abist, attribution kept.)
2. Load a `.ply`, project gaussians (CPU preprocess OK), draw as additive quads with the poly+WSR
   shader → an image on screen.
3. **Offscreen → PNG + PSNR-diff harness vs the PyTorch reference** (§5). Get a first numeric match on a
   trivial hand-made scene (a few gaussians). **This gates correctness — do it early.**
4. (a) A1 qualitative: standard `.ply` via WSR — capture/observe the occlusion behavior.
5. (b) Exporter in tt-splat + loader here → render our trained ficus model; diff vs tt-splat's held-out
   render.
6. Tiling + bounding-radius cull + fixed-K binning → A3 scale/quality exploration.

When in doubt about the math, **match the PyTorch output numerically (§5)** rather than guessing.

---

## WASM / browser build (works; verified in Chrome)

The viewer runs in a WebGPU browser. Build + serve:

```
cargo build --lib --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/tt_splat_viewer.wasm --out-dir web --target web --no-typescript
python3 -m http.server 8080 --bind 127.0.0.1 --directory web      # then open http://localhost:8080
```

- Uses the **WebGPU** backend (Cargo wasm deps drop the `webgl` feature) because the WSR accumulator
  is `Rgba16Float` with additive blending, which WebGL2 can't reliably render+blend. Needs Chrome/Edge
  or Safari 17.4+.
- `web/index.html` (tracked) contains a **compat shim** that strips the `maxInterStageShaderComponents`
  limit from `GPUAdapter.requestDevice` — wgpu 0.20 still sends it but current Chrome removed it from
  the spec and rejects the call. Real fix later: bump wgpu. The shim also mirrors console errors onto
  the page (handy when the only feedback is a screenshot).
- `web/*.wasm` and `web/*.js` are generated (gitignored); regenerate with the two commands above.
- On this headless dev box, serving over Tailscale works: `sudo tailscale serve --bg 8080` →
  `https://<host>.<tailnet>.ts.net/` (HTTPS satisfies WebGPU's secure-context requirement).
- The default scene (no `.ply` arg, and always on WASM) is `scene::demo_scene()` — a procedural
  ~1200-gaussian colored sphere (no asset file needed).

## Dual-pane comparison view (branch `dual-pane-viewer`)

The window/canvas is split into two synced panes: **left = WSR** (the tt-splat/BH method, `Renderer`),
**right = standard 3DGS** (`GsRenderer`: `exp(−Q/2)`, depth-sorted "over" alpha — mirrors
`spike/arms.render_D`, with **view-dependent SH deg-3 color** from a real `.ply`'s `f_rest_*`).
Both share one `Orbit` camera
(automatic sync). Each pane renders into its own `Rgba8Unorm` target; `blit.wgsl` places them
side-by-side with a divider.

- `scene::preprocess` gained a `radius_sigma` arg ([`WSR_SIGMAS`]=2 / [`GS_SIGMAS`]=3 footprint);
  `preprocess_sorted` returns far→near order for painter's-order alpha. `InstanceRaw._pad` → `depth`.
- Drag-and-drop is per-pane: the drop x-position (JS `load_ply_into(pane,bytes)` / native cursor)
  picks left or right. Either pane works empty/independently; default is the demo sphere in both.
- **Background `b`-toggle** (black / white / slate; `BG_PRESETS`). White = tt-splat's training bg.
  **Route-B `.ply` background gotcha (verified):** `spike/plyio.save_ply` does NOT store `w_b`/`c_b`,
  and route-B models train with `c_b`=white, `w_b`=softplus(-3)≈0.049 (`spike/model.py`). The viewer
  defaults to black, so the model's white "empty-space" gaussians show as a white haze — press `b`
  for white and they vanish. The WSR *method* is correct: `validation/verify_routeb.py` exports a
  scene via tt-splat's own `plyio` and PSNR-matches the viewer's WSR to tt-splat `render()` at
  **61.92 dB** (black bg both sides). Not a renderer bug — a missing-background-metadata data issue.
- Offscreen check: `offscreen --gs --demo out.png` renders the 3DGS pane headlessly (occlusion looks
  solid vs WSR's averaged look). `offscreen --dual out.png` renders the **full two-pane path**
  (two `Pane`s + blit, like the window) so the dual view is verifiable on the headless box without a
  browser. WSR PSNR vs oracle unchanged (50.39 dB) after the refactor.
- Default sample (no `.ply`): `scene::synthetic_scene()` — the 3 single-color gaussians, in both panes.
- **Camera**: `Orbit` is quaternion-based (`orientation: Quat`), so drag-rotation is **unlimited** in
  every direction (yaw about world-up, pitch about the current right axis; no pole clamp). `set_angles`
  reconstructs it from yaw/pitch for the offscreen `--orbit`/`--dual` renders.
- **XYZ gizmo is external** (in the page, not in the panes): `view_basis()` exports the camera basis
  (9 floats, column-major = world X/Y/Z in camera space), and `web/index.html` draws a small 2D
  orientation gizmo that tracks the shared camera. (An earlier in-pane wgpu gizmo was removed per
  user preference.)
- `web/index.html` has `Cache-Control: no-store` to avoid stale-wasm confusion after a redeploy; still
  re-commit `web/*.wasm,*.js` after code changes.
- **SH (deg-3) color** for the 3DGS pane: `scene::parse_ply` reads `f_rest_*` (channel-major) into
  `Gaussian.sh`; `preprocess(..., eval_sh=true)` evaluates `eval_sh_color` at the view dir (WSR passes
  `false` → DC only, so the tt-splat oracle still matches). Cross-checked vs an independent eval:
  `validation/sh_check.py` (1-gaussian known `f_rest`) → center pixel max|Δ| = 0.002.
  ```
  ../tt-splat/.venv/bin/python validation/sh_check.py
  cargo run --bin offscreen -- --gs validation/model_sh.ply validation/sh.png validation/view_sh.json
  ../tt-splat/.venv/bin/python validation/sh_check.py --compare
  ```

## Build prerequisites (dev box, one-time)

The native build needs a C toolchain + the Linux windowing/Vulkan dev libs. Rust is installed
(rustup, stable). Still missing — install with (needs sudo):

```
sudo apt-get install -y build-essential pkg-config \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev libwayland-dev libxkbcommon-dev \
  mesa-vulkan-drivers vulkan-tools libvulkan-dev
```

For the WASM path: `rustup target add wasm32-unknown-unknown` + `cargo install trunk wasm-bindgen-cli`.

## Progress log

- Bootstrap commit: handoff doc + license + scaffolding.
- Doc reorg: `HANDOFF.md` → this `CLAUDE.md` (git-tracked); README de-jargoned (dropped "(B) route").
- **M1 (skeleton):** `src/{main,lib}.rs` — winit 0.29 + wgpu 0.20 window, surface, clear/redraw loop;
  native + WASM (`#[wasm_bindgen(start)]`) entry paths. Cargo set up.
- **M2 (renderer):** `src/scene.rs` (CPU `project_ewa`/`cov3d`/quat + `color_from_dc` + synthetic
  3-gaussian scene), `src/shader.wgsl` (poly-splat Q/w_geo + WSR divide), two-pass additive→composite
  pipeline in `lib.rs`.
- **M3 (offscreen + PSNR harness) — DONE & PASSING.** `src/offscreen.rs` + `src/bin/offscreen.rs`
  render `scene.json` headlessly to PNG; `validation/oracle.py` writes the shared `scene.json` and the
  PyTorch arm-A reference; `validation/psnr.py` diffs them. **Result: PSNR 50.39 dB, max|Δ| = 1/255**
  (i.e. bit-accurate up to 8-bit quantization — the 50 dB target is saturated by the uint8 output, so
  no convention bug). Reproduce:
  ```
  ../tt-splat/.venv/bin/python validation/oracle.py
  cargo run --bin offscreen -- validation/scene.json validation/rust.png
  ../tt-splat/.venv/bin/python validation/psnr.py validation/oracle.png validation/rust.png
  ```
  Dev box is **headless** (no DISPLAY) but has an RTX 3090 + Vulkan, so the offscreen path works; the
  windowed `cargo run` needs a display (untested here).
- **M4 (`.ply` viewer) — DONE & PASSING.** `scene::load_ply` reads standard INRIA-3DGS `.ply`
  (binary-LE/ascii, properties by name; skips normals/`f_rest`). `scene::Orbit` auto-frames a scene
  and drives an orbit camera; the windowed path (`run()`) loads a `.ply` from argv and supports
  drag-to-orbit + scroll-zoom (re-runs the CPU preprocess per move). `validation/export_ply.py`
  exports a tt-splat `GaussianModel(gt=True)` to a `.ply` + oracle-matched `view.json` + arm-A render.
  **Result: 2000 gaussians @ 512², PSNR 67.10 dB, max|Δ| = 1/255.** The orbit/auto-frame code is also
  exercised headlessly via `offscreen ... --orbit <yaw> <pitch>` (the interactive window itself is
  untested here — the dev box has no display).
  ```
  ../tt-splat/.venv/bin/python validation/export_ply.py 2000
  cargo run --bin offscreen -- validation/model.ply validation/rust_ply.png validation/view.json
  ../tt-splat/.venv/bin/python validation/psnr.py validation/oracle_ply.png validation/rust_ply.png
  ```
- NOT yet done: a *real captured* 3DGS scene (download one to point the viewer at — that's the true
  A1 occlusion demo; the gt export above is synthetic and uniform), the faithful tt-splat exporter for
  our trained gaussians (M5), tiling/bounding-cull/fixed-K for scale (M6). To push PSNR past the 8-bit
  ceiling, diff in float (dump f32 from both sides).
- Cargo.lock is gitignored (minimal-tracking policy); flip that if you want reproducible bin builds.
- No abist code is actually vendored yet — the plumbing here is original. Keep the attribution clause
  ready for if/when a file is genuinely adapted.
