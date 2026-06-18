# tt-splat-viewer — Handoff for the next Claude Code session

You are starting work in a **fresh, empty repo**. This file is your full context — the prior
session that created it is not available. Read it top to bottom before writing code.

---

## 0. What you are building (TL;DR)

A **from-scratch Rust + wgpu + WGSL** interactive viewer for the **"(B) route"** Gaussian-splatting
renderer: **poly-splat + Weighted Sum Rendering (WSR)** — order-independent, **no depth sort, no
transcendentals**. Native (desktop) + WASM (browser).

**Purpose (be honest about scope):** this is a *reference / visual de-risk / demo* tool. It runs on a
normal GPU via wgpu. **It is NOT the performance artifact** — the parent project's perf thesis is about
running on Tenstorrent Blackhole silicon, which this viewer does not touch. **No perf/$ claim ever comes
from this repo.** Its job is to let us *see* the (B) model on real scenes and de-risk its known
weaknesses (below).

---

## 1. Parent project context (`tt-splat`)

- Sibling repo on this machine: **`../tt-splat`** (private, `github.com/kinginu/tt-splat`). You can read it.
- tt-splat maps 3D Gaussian Splatting onto **Tenstorrent Blackhole**. The "(B) route" swaps standard
  3DGS's sorted-alpha compositing for a hardware-friendly formulation:
  - **No depth sort** → Weighted Sum Rendering (WSR), order-independent.
  - **No transcendentals** → poly-splat weight `(1−Q/k)₊²` instead of `exp(−Q/2)`.
  - **Matmul-shaped** → the quadratic form `Q = Φ·θᵀ` is an exact GEMM.
- Phase 1 (hardware-independent) is **complete**: the (B) math is de-risked (depth-free WSR ≈ parity
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
`C = num/den`.)

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
   (`© 2026 株式会社アビスト イノベーションセンター`). See `LICENSE` / README attribution.
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
- **(b) Faithful (B) viewer (the real artifact):** consume **our** trained gaussians exported from
  tt-splat — `mean3d`, our conic/θ, `color_dc`, `opacity_raw`, `w_b`, `c_b`. This needs a small
  exporter on the tt-splat side and a matching loader here. This is what actually shows the (B) model.

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

## 6. Suggested milestones (commit at each)

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

---

## 7. Logistics

- This repo: `github.com/kinginu/tt-splat-viewer` (private), MIT.
- Parent reference (readable, same machine): `../tt-splat`. Start with `spike/{render,arms,forward,geometry}.py`.
- Git identity is already set (`kinginu`). Commit/push to `main` when the user asks.
- Keep abist's MIT copyright on any lifted plumbing; our `LICENSE` covers new code.
- When in doubt about the math, **match the PyTorch output numerically (§5)** rather than guessing.
