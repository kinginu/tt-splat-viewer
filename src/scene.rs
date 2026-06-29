//! CPU-side scene + geometry preprocess. Mirrors `../tt-splat/spike/{geometry,camera,forward}.py`.
//!
//! The per-gaussian "G-setup" (quat→rotation, 3D cov, EWA projection to a 2D mean + conic) is cheap
//! and done on the CPU; the per-pixel hot path (Q, poly-splat, WSR blend) lives in `shader.wgsl`.
//! See `CLAUDE.md` §2. Numbers here must match the oracle — verify via the PSNR harness (§5), not by eye.

use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Quat, Vec3};

/// Poly-splat width constant `k` (spike uses 4.0). Also referenced as a `const` in the shader.
pub const K: f32 = 4.0;
/// EWA low-pass added to the 2D covariance diagonal (`blur_eps` in `project_ewa`).
pub const BLUR_EPS: f32 = 0.3;
/// Near plane; gaussians with depth ≤ near are culled (`keep`).
pub const NEAR: f32 = 0.2;
/// SH degree-0 normalization constant `C0` (matches `spike/sh.py`).
pub const SH_C0: f32 = 0.282_094_79;

/// One gaussian in the tt-splat parameterization (raw / pre-activation where the oracle is).
#[derive(Clone, Copy, Debug)]
pub struct Gaussian {
    pub mean: Vec3,
    /// `log` scale (the oracle stores `log_scales`; we `exp` it in `cov3d`).
    pub log_scale: Vec3,
    /// Quaternion in `[w, x, y, z]` order (normalized internally).
    pub quat: [f32; 4],
    /// SH degree-0 DC color coefficient (`color_dc`); final color = `clamp(0.5 + C0·dc, 0)`.
    pub color_dc: Vec3,
    /// Pre-sigmoid opacity (`opacity_raw`).
    pub opacity_raw: f32,
    /// Higher-order SH color coefficients (deg 1-3, 15 per channel, channel-major: R×15,G×15,B×15)
    /// from a standard 3DGS `.ply`'s `f_rest_*`. `None` for DC-only sources (demo, tt-splat exports).
    /// Used only by the 3DGS pane; WSR ignores it (the tt-splat model is DC-only).
    pub sh: Option<[f32; 45]>,
}

/// Learnable WSR background: `C = (Σ w·color + w_b·c_b) / (Σ w + w_b)`.
#[derive(Clone, Copy, Debug)]
pub struct Background {
    /// Already-positive weight (oracle applies `softplus` to `w_b_raw`; do it before constructing).
    pub w_b: f32,
    pub c_b: Vec3,
}

/// OpenCV pinhole camera (world→cam `R_v`, `t_v`) + intrinsics + image size. Mirrors `camera.Camera`.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub r_v: Mat3,
    pub t_v: Vec3,
    pub fx: f32,
    pub fy: f32,
    pub cx: f32,
    pub cy: f32,
    pub width: u32,
    pub height: u32,
}

impl Camera {
    /// Build directly from a world→cam rotation `r_v` (row-major 9) + translation `t_v`.
    pub fn from_rt(r_v: [f32; 9], t_v: [f32; 3], fx: f32, fy: f32, cx: f32, cy: f32, width: u32, height: u32) -> Self {
        // r_v is row-major; glam Mat3::from_cols wants columns.
        let m = Mat3::from_cols(
            Vec3::new(r_v[0], r_v[3], r_v[6]),
            Vec3::new(r_v[1], r_v[4], r_v[7]),
            Vec3::new(r_v[2], r_v[5], r_v[8]),
        );
        Self {
            r_v: m,
            t_v: Vec3::from_array(t_v),
            fx,
            fy,
            cx,
            cy,
            width,
            height,
        }
    }

    /// `look_at` in the OpenCV convention (camera looks down +z, +y down). Mirrors `look_at_opencv`.
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3, fx: f32, fy: f32, width: u32, height: u32) -> Self {
        let z = (target - eye).normalize();
        let x = up.cross(z).normalize();
        let y = z.cross(x);
        // rows = cam axes -> world->cam rotation
        let r_v = Mat3::from_cols(x, y, z).transpose();
        let t_v = -(r_v * eye);
        Self {
            r_v,
            t_v,
            fx,
            fy,
            cx: width as f32 / 2.0,
            cy: height as f32 / 2.0,
            width,
            height,
        }
    }
}

/// Per-gaussian instance data uploaded to the GPU after CPU preprocess. One additive quad each.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InstanceRaw {
    /// 2D mean in pixel coords.
    pub mu2d: [f32; 2],
    /// Axis-aligned quad half-extent in pixels (the poly-splat footprint where `Q < k`).
    pub half_extent: [f32; 2],
    /// Conic = inverse 2D covariance `(a, b, c) = (M00, M01, M11)`.
    pub conic: [f32; 3],
    /// Pre-multiplied opacity `o = sigmoid(opacity_raw)`.
    pub opacity: f32,
    /// Final RGB color `clamp(0.5 + C0·dc, 0)`.
    pub color: [f32; 3],
    /// Camera-space depth (z). Not a vertex attribute — used to depth-sort for the 3DGS renderer.
    pub depth: f32,
}

/// Footprint radius (in standard deviations) for the poly-splat WSR quad: `sqrt(k)` so the quad
/// exactly covers the `Q < k` support. `half_extent = WSR_SIGMAS · sqrt(Σ_ii)`.
pub const WSR_SIGMAS: f32 = 2.0; // sqrt(K) with K = 4
/// Footprint radius for the 3DGS `exp(−Q/2)` quad — 3σ captures the exponential tail.
pub const GS_SIGMAS: f32 = 3.0;

/// Raw per-gaussian data uploaded to the GPU **once** (not per frame) — the vertex shader does the
/// EWA projection itself. Four `vec4`s (64 B, 16-byte aligned) so it works as instance attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GaussianRaw {
    pub mean_opacity: [f32; 4],  // mean.xyz, opacity_raw
    pub log_scale: [f32; 4],     // log_scale.xyz, _
    pub quat: [f32; 4],          // w, x, y, z
    pub color_dc: [f32; 4],      // color_dc.xyz, _
}

/// Per-frame camera uniform consumed by the GPU-projection vertex shader (96 B, six `vec4`s).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CamUniform {
    pub r_v0: [f32; 4], // columns of R_v (world→cam rotation); .xyz used
    pub r_v1: [f32; 4],
    pub r_v2: [f32; 4],
    pub t_v_near: [f32; 4], // t_v.xyz, near
    pub intr: [f32; 4],     // fx, fy, cx, cy
    pub misc: [f32; 4],     // viewport.x, viewport.y, radius_sigma², blur_eps
}

fn raw_of(g: &Gaussian) -> GaussianRaw {
    GaussianRaw {
        mean_opacity: [g.mean.x, g.mean.y, g.mean.z, g.opacity_raw],
        log_scale: [g.log_scale.x, g.log_scale.y, g.log_scale.z, 0.0],
        quat: g.quat,
        color_dc: [g.color_dc.x, g.color_dc.y, g.color_dc.z, 0.0],
    }
}

/// Repack gaussians into GPU-ready raw form (no projection — that happens on the GPU).
pub fn to_raw(gaussians: &[Gaussian]) -> Vec<GaussianRaw> {
    gaussians.iter().map(raw_of).collect()
}

/// Like [`to_raw`] but ordered far→near for the 3DGS painter's-order pane. Only the cheap camera-space
/// depth (`(R_v·mean + t_v).z`) is computed per gaussian — not the full projection (that's on the GPU).
pub fn sorted_raw(gaussians: &[Gaussian], cam: &Camera) -> Vec<GaussianRaw> {
    let keys: Vec<f32> = gaussians.iter().map(|g| (cam.r_v * g.mean + cam.t_v).z).collect();
    let mut idx: Vec<u32> = (0..gaussians.len() as u32).collect();
    idx.sort_unstable_by(|&a, &b| keys[b as usize].total_cmp(&keys[a as usize])); // descending
    idx.iter().map(|&i| raw_of(&gaussians[i as usize])).collect()
}

/// Build the per-frame camera uniform for the GPU-projection shader. `radius_sigma` sets the quad
/// footprint ([`WSR_SIGMAS`] / [`GS_SIGMAS`]); the shader uses `radius_sigma²·Σ_ii` for the half-extent.
pub fn cam_uniform(cam: &Camera, radius_sigma: f32) -> CamUniform {
    let c = cam.r_v.to_cols_array(); // column-major: [c0, c1, c2]
    CamUniform {
        r_v0: [c[0], c[1], c[2], 0.0],
        r_v1: [c[3], c[4], c[5], 0.0],
        r_v2: [c[6], c[7], c[8], 0.0],
        t_v_near: [cam.t_v.x, cam.t_v.y, cam.t_v.z, NEAR],
        intr: [cam.fx, cam.fy, cam.cx, cam.cy],
        misc: [cam.width as f32, cam.height as f32, radius_sigma * radius_sigma, BLUR_EPS],
    }
}

/// `quat_to_rotmat` (`[w,x,y,z]`, normalized). Mirrors `geometry.quat_to_rotmat`.
fn quat_to_rotmat(q: [f32; 4]) -> Mat3 {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt().max(1e-12);
    let (w, x, y, z) = (q[0] / n, q[1] / n, q[2] / n, q[3] / n);
    // Row-major in the oracle; glam Mat3::from_cols takes columns, so transpose by supplying columns.
    Mat3::from_cols_array(&[
        1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + w * z),       2.0 * (x * z - w * y),
        2.0 * (x * y - w * z),       1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + w * x),
        2.0 * (x * z + w * y),       2.0 * (y * z - w * x),       1.0 - 2.0 * (x * x + y * y),
    ])
}

/// `Sigma3 = R diag(scale²) Rᵀ` with `scale = exp(log_scale)`. Mirrors `geometry.cov3d`.
fn cov3d(log_scale: Vec3, r: Mat3) -> Mat3 {
    let s = Vec3::new(log_scale.x.exp(), log_scale.y.exp(), log_scale.z.exp());
    let s2 = s * s;
    // R * diag(s2) * R^T
    let rs = Mat3::from_cols(r.x_axis * s2.x, r.y_axis * s2.y, r.z_axis * s2.z);
    rs * r.transpose()
}

// Real-SH basis constants (gsplat / INRIA convention; C0..C2 match spike/sh.py, C3 is the deg-3 set).
const SH_C1: f32 = 0.488_602_5;
const SH_C2: [f32; 5] = [1.092_548_4, -1.092_548_4, 0.315_391_57, -1.092_548_4, 0.546_274_2];
const SH_C3: [f32; 7] = [
    -0.590_043_6, 2.890_611_4, -0.457_045_8, 0.373_176_33, -0.457_045_8, 1.445_305_7, -0.590_043_6,
];

/// Evaluate standard 3DGS view-dependent color: `clamp(0.5 + Σ Y_i(dir)·coeff_i, 0)` per channel.
/// `dc` is the DC term, `rest` the 15 higher coeffs per channel (channel-major), `dir` view direction.
fn eval_sh_color(dc: Vec3, rest: &[f32; 45], dir: Vec3) -> [f32; 3] {
    let d = dir.normalize_or_zero();
    let (x, y, z) = (d.x, d.y, d.z);
    let (xx, yy, zz) = (x * x, y * y, z * z);
    let (xy, yz, xz) = (x * y, y * z, x * z);
    let dc = [dc.x, dc.y, dc.z];
    let mut out = [0.0f32; 3];
    for c in 0..3 {
        let s = |k: usize| rest[c * 15 + (k - 1)]; // SH index k = 1..15
        let mut r = SH_C0 * dc[c];
        r += -SH_C1 * y * s(1) + SH_C1 * z * s(2) - SH_C1 * x * s(3);
        r += SH_C2[0] * xy * s(4)
            + SH_C2[1] * yz * s(5)
            + SH_C2[2] * (2.0 * zz - xx - yy) * s(6)
            + SH_C2[3] * xz * s(7)
            + SH_C2[4] * (xx - yy) * s(8);
        r += SH_C3[0] * y * (3.0 * xx - yy) * s(9)
            + SH_C3[1] * xy * z * s(10)
            + SH_C3[2] * y * (4.0 * zz - xx - yy) * s(11)
            + SH_C3[3] * z * (2.0 * zz - 3.0 * xx - 3.0 * yy) * s(12)
            + SH_C3[4] * x * (4.0 * zz - xx - yy) * s(13)
            + SH_C3[5] * z * (xx - yy) * s(14)
            + SH_C3[6] * x * (xx - 3.0 * yy) * s(15);
        out[c] = (r + 0.5).max(0.0);
    }
    out
}

/// Project every gaussian to an `InstanceRaw`; culled gaussians (`keep == false`) are dropped.
/// Mirrors `geometry.project_ewa` + `forward.color_from_dc` + the opacity sigmoid. `radius_sigma`
/// sets the quad footprint in σ ([`WSR_SIGMAS`] for poly-splat, [`GS_SIGMAS`] for 3DGS `exp`).
/// With `eval_sh`, gaussians that carry `sh` coeffs get view-dependent color (the 3DGS pane); WSR
/// passes `false` for DC-only color (matching the tt-splat oracle).
pub fn preprocess(gaussians: &[Gaussian], cam: &Camera, radius_sigma: f32, eval_sh: bool) -> Vec<InstanceRaw> {
    let cam_center = -(cam.r_v.transpose() * cam.t_v); // world-space camera position for view dirs
    let mut out = Vec::with_capacity(gaussians.len());
    for g in gaussians {
        let r = quat_to_rotmat(g.quat);
        let cov_w = cov3d(g.log_scale, r);

        let mu_cam = cam.r_v * g.mean + cam.t_v;
        let depth = mu_cam.z;
        if depth <= NEAR {
            continue; // keep = depth > near
        }
        let z = depth.max(NEAR);

        let mu2d = [cam.fx * mu_cam.x / z + cam.cx, cam.fy * mu_cam.y / z + cam.cy];

        // Sigma_cam = R_v Sigma3 R_v^T
        let sigma_cam = cam.r_v * cov_w * cam.r_v.transpose();

        // EWA Jacobian J (2x3): [[fx/z, 0, -fx*x/z²], [0, fy/z, -fy*y/z²]]
        let j = [
            [cam.fx / z, 0.0, -cam.fx * mu_cam.x / (z * z)],
            [0.0, cam.fy / z, -cam.fy * mu_cam.y / (z * z)],
        ];
        // Sigma2d = J Sigma_cam J^T  (2x2)
        let sc = sigma_cam.to_cols_array_2d(); // sc[col][row]; symmetric so indexing order is fine
        let mut sigma2d = [[0.0f32; 2]; 2];
        for a in 0..2 {
            for b in 0..2 {
                let mut acc = 0.0;
                for i in 0..3 {
                    for k in 0..3 {
                        acc += j[a][i] * sc[i][k] * j[b][k];
                    }
                }
                sigma2d[a][b] = acc;
            }
        }
        // low-pass on the covariance diagonal
        sigma2d[0][0] += BLUR_EPS;
        sigma2d[1][1] += BLUR_EPS;

        let (s00, s01, s11) = (sigma2d[0][0], sigma2d[0][1], sigma2d[1][1]);
        let det = (s00 * s11 - s01 * s01).max(1e-12);
        let conic = [s11 / det, -s01 / det, s00 / det]; // (a, b, c)

        // Footprint half-extent: `radius_sigma` standard deviations along each axis (σ_ii = sqrt(Σ_ii)).
        let half_extent = [radius_sigma * s00.sqrt(), radius_sigma * s11.sqrt()];

        let opacity = 1.0 / (1.0 + (-g.opacity_raw).exp());
        let color = match (eval_sh, &g.sh) {
            (true, Some(rest)) => eval_sh_color(g.color_dc, rest, g.mean - cam_center),
            _ => [
                (0.5 + SH_C0 * g.color_dc.x).max(0.0),
                (0.5 + SH_C0 * g.color_dc.y).max(0.0),
                (0.5 + SH_C0 * g.color_dc.z).max(0.0),
            ],
        };

        out.push(InstanceRaw {
            mu2d,
            half_extent,
            conic,
            opacity,
            color,
            depth,
        });
    }
    out
}

/// Like [`preprocess`] but sorted back-to-front (farthest first) for painter's-order alpha
/// compositing — the draw order the standard 3DGS renderer needs.
pub fn preprocess_sorted(gaussians: &[Gaussian], cam: &Camera, radius_sigma: f32, eval_sh: bool) -> Vec<InstanceRaw> {
    let mut out = preprocess(gaussians, cam, radius_sigma, eval_sh);
    out.sort_by(|a, b| b.depth.total_cmp(&a.depth)); // descending depth = far → near
    out
}

/// Load a scene + camera from the shared `scene.json` used by the PSNR validation harness.
/// Both this loader and `validation/oracle.py` read the same file so the inputs match exactly.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_scene_json(path: &std::path::Path) -> std::io::Result<(Vec<Gaussian>, Background, Camera)> {
    #[derive(serde::Deserialize)]
    struct CamJson { r_v: [f32; 9], t_v: [f32; 3], fx: f32, fy: f32, cx: f32, cy: f32 }
    #[derive(serde::Deserialize)]
    struct BgJson { w_b: f32, c_b: [f32; 3] }
    #[derive(serde::Deserialize)]
    struct GaussJson {
        mean: [f32; 3],
        log_scale: [f32; 3],
        quat: [f32; 4],
        color_dc: [f32; 3],
        opacity_raw: f32,
    }
    #[derive(serde::Deserialize)]
    struct SceneJson {
        width: u32,
        height: u32,
        camera: CamJson,
        background: BgJson,
        gaussians: Vec<GaussJson>,
    }

    let text = std::fs::read_to_string(path)?;
    let s: SceneJson = serde_json::from_str(&text)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let cam = Camera::from_rt(
        s.camera.r_v, s.camera.t_v, s.camera.fx, s.camera.fy, s.camera.cx, s.camera.cy, s.width, s.height,
    );
    let bg = Background { w_b: s.background.w_b, c_b: Vec3::from_array(s.background.c_b) };
    let gaussians = s
        .gaussians
        .into_iter()
        .map(|g| Gaussian {
            mean: Vec3::from_array(g.mean),
            log_scale: Vec3::from_array(g.log_scale),
            quat: g.quat,
            color_dc: Vec3::from_array(g.color_dc),
            opacity_raw: g.opacity_raw,
            sh: None,
        })
        .collect();
    Ok((gaussians, bg, cam))
}

/// Load a camera + background (no gaussians) from a `view.json` — used to render a `.ply` with a
/// fixed, oracle-matched camera in the PSNR harness.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_view_json(path: &std::path::Path) -> std::io::Result<(Background, Camera)> {
    #[derive(serde::Deserialize)]
    struct CamJson { r_v: [f32; 9], t_v: [f32; 3], fx: f32, fy: f32, cx: f32, cy: f32 }
    #[derive(serde::Deserialize)]
    struct BgJson { w_b: f32, c_b: [f32; 3] }
    #[derive(serde::Deserialize)]
    struct ViewJson { width: u32, height: u32, camera: CamJson, background: BgJson }

    let text = std::fs::read_to_string(path)?;
    let v: ViewJson = serde_json::from_str(&text)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let cam = Camera::from_rt(
        v.camera.r_v, v.camera.t_v, v.camera.fx, v.camera.fy, v.camera.cx, v.camera.cy, v.width, v.height,
    );
    Ok((Background { w_b: v.background.w_b, c_b: Vec3::from_array(v.background.c_b) }, cam))
}

/// Default WSR background for a standard `.ply` (which carries no background): dim, near-zero weight.
pub fn default_background() -> Background {
    Background { w_b: 0.02, c_b: Vec3::ZERO }
}

/// Read a `.ply` from disk and parse it (native convenience over [`parse_ply`]).
#[cfg(not(target_arch = "wasm32"))]
pub fn load_ply(path: &std::path::Path) -> std::io::Result<Vec<Gaussian>> {
    parse_ply(&std::fs::read(path)?)
}

/// Parse a standard INRIA-3DGS `.ply` from bytes (binary-little-endian or ascii). Reads vertex
/// properties by name: `x y z`, `scale_0..2` (log), `rot_0..3` (quaternion `w,x,y,z`), `f_dc_0..2`
/// (SH-deg0 color), `opacity` (pre-sigmoid), and `f_rest_*` (higher-order SH, used by the 3DGS pane;
/// WSR ignores it). Normals are skipped. Available on all targets (no filesystem) for WASM drops.
pub fn parse_ply(bytes: &[u8]) -> std::io::Result<Vec<Gaussian>> {
    use std::io::{Error, ErrorKind};

    let marker = b"end_header\n";
    let header_end = bytes
        .windows(marker.len())
        .position(|w| w == marker)
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "no end_header"))?;
    let data_start = header_end + marker.len();
    let header = String::from_utf8_lossy(&bytes[..header_end]);

    let type_size = |t: &str| -> Option<usize> {
        Some(match t {
            "char" | "uchar" | "int8" | "uint8" => 1,
            "short" | "ushort" | "int16" | "uint16" | "half" | "float16" => 2,
            "int" | "uint" | "int32" | "uint32" | "float" | "float32" => 4,
            "double" | "float64" => 8,
            _ => return None,
        })
    };

    let mut is_ascii = false;
    let mut is_be = false;
    let mut count = 0usize;
    // (name, byte size) for the `vertex` element only (scoped — other elements must not inflate stride).
    let mut props: Vec<(String, usize)> = Vec::new();
    let mut cur = String::new();
    let mut unknown: Vec<String> = Vec::new();
    for line in header.lines() {
        let tok: Vec<&str> = line.split_whitespace().collect();
        match tok.as_slice() {
            ["format", fmt, ..] => {
                is_ascii = fmt.starts_with("ascii");
                is_be = fmt.starts_with("binary_big");
            }
            ["element", name, n] => {
                cur = name.to_string();
                if *name == "vertex" {
                    count = n.parse().unwrap_or(0);
                }
            }
            ["property", ty, name] if cur == "vertex" => {
                let sz = type_size(ty).unwrap_or_else(|| {
                    if !unknown.iter().any(|u| u == ty) {
                        unknown.push(ty.to_string());
                    }
                    4
                });
                props.push((name.to_string(), sz));
            }
            _ => {}
        }
    }
    if is_be {
        return Err(Error::new(ErrorKind::InvalidData, "binary_big_endian .ply not supported"));
    }

    let idx = |name: &str| props.iter().position(|(n, _)| n == name);
    let need = ["x", "y", "z", "scale_0", "scale_1", "scale_2", "rot_0", "rot_1", "rot_2", "rot_3",
        "f_dc_0", "f_dc_1", "f_dc_2", "opacity"];
    let mut col = std::collections::HashMap::new();
    for name in need {
        let i = idx(name).ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("missing property {name}")))?;
        col.insert(name, i);
    }

    // Per-property byte offset within a vertex record (binary).
    let mut offset = Vec::with_capacity(props.len());
    let mut acc = 0usize;
    for (_, sz) in &props {
        offset.push(acc);
        acc += sz;
    }
    let stride = acc;

    // Higher-order SH (`f_rest_*`), in index order — standard 3DGS stores them channel-major.
    let mut rest_cols: Vec<(u32, usize)> = props
        .iter()
        .enumerate()
        .filter_map(|(i, (n, _))| {
            n.strip_prefix("f_rest_").and_then(|s| s.parse::<u32>().ok()).map(|k| (k, i))
        })
        .collect();
    rest_cols.sort_by_key(|(k, _)| *k);
    let rest_cols: Vec<usize> = rest_cols.into_iter().map(|(_, i)| i).collect();
    // Valid only if divisible into 3 channels; per-channel count clamped to deg-3 (15).
    let per_ch = if !rest_cols.is_empty() && rest_cols.len() % 3 == 0 {
        (rest_cols.len() / 3).min(15)
    } else {
        0
    };
    let total_per_ch = rest_cols.len() / 3; // file's actual per-channel stride (may exceed 15)

    let build_sh = |read: &dyn Fn(usize) -> f32| -> Option<[f32; 45]> {
        if per_ch == 0 {
            return None;
        }
        let mut sh = [0.0f32; 45];
        for c in 0..3 {
            for j in 0..per_ch {
                sh[c * 15 + j] = read(rest_cols[c * total_per_ch + j]);
            }
        }
        Some(sh)
    };

    let g_at = |vals: &dyn Fn(&str) -> f32, sh: Option<[f32; 45]>| Gaussian {
        mean: Vec3::new(vals("x"), vals("y"), vals("z")),
        log_scale: Vec3::new(vals("scale_0"), vals("scale_1"), vals("scale_2")),
        quat: [vals("rot_0"), vals("rot_1"), vals("rot_2"), vals("rot_3")],
        color_dc: Vec3::new(vals("f_dc_0"), vals("f_dc_1"), vals("f_dc_2")),
        opacity_raw: vals("opacity"),
        sh,
    };

    let mut out = Vec::with_capacity(count);
    if is_ascii {
        let text = String::from_utf8_lossy(&bytes[data_start..]);
        for line in text.lines().filter(|l| !l.trim().is_empty()).take(count) {
            let nums: Vec<f32> = line.split_whitespace().filter_map(|t| t.parse().ok()).collect();
            let read = |ci: usize| nums[ci];
            let vals = |name: &str| read(col[name]);
            let sh = build_sh(&read);
            out.push(g_at(&vals, sh));
        }
    } else {
        // Up-front size check with a precise, actionable message (vs. a per-vertex one mid-loop).
        let need = data_start + count.saturating_mul(stride);
        if need > bytes.len() {
            let extra = if unknown.is_empty() {
                String::new()
            } else {
                format!(" — unrecognized property types {unknown:?} were sized as 4 bytes, which likely \
                         made the stride wrong (compressed/half-float .ply?)")
            };
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "ply body truncated: expected {need} bytes ({count} verts × {stride}-byte record \
                     + {data_start} header, {} properties) but the file is {} bytes{extra}",
                    props.len(),
                    bytes.len()
                ),
            ));
        }
        for i in 0..count {
            let base = data_start + i * stride;
            if base + stride > bytes.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "ply body truncated"));
            }
            let read = |ci: usize| -> f32 {
                let o = base + offset[ci];
                f32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]])
            };
            let vals = |name: &str| read(col[name]);
            let sh = build_sh(&read);
            out.push(g_at(&vals, sh));
        }
    }
    Ok(out)
}

/// A trackball/orbit camera. Orientation is a quaternion (not yaw/pitch Euler angles) so rotation is
/// **unlimited in every direction** — no pole clamp, you can roll the model right over the top.
#[derive(Clone, Copy, Debug)]
pub struct Orbit {
    pub target: Vec3,
    pub orientation: Quat,
    pub radius: f32,
    pub fovy: f32,
}

impl Orbit {
    /// Frame a gaussian set: centroid as target, radius from the bounding sphere + fov.
    pub fn frame(gaussians: &[Gaussian]) -> Self {
        let fovy = 60f32.to_radians();
        if gaussians.is_empty() {
            return Orbit { target: Vec3::ZERO, orientation: Quat::IDENTITY, radius: 5.0, fovy };
        }
        let centroid = gaussians.iter().map(|g| g.mean).sum::<Vec3>() / gaussians.len() as f32;
        let bound = gaussians
            .iter()
            .map(|g| (g.mean - centroid).length())
            .fold(0.0f32, f32::max)
            .max(1e-3);
        // distance so the bounding sphere fits the vertical fov, with margin.
        let radius = 1.4 * bound / (fovy * 0.5).sin();
        Orbit { target: centroid, orientation: Quat::IDENTITY, radius, fovy }
    }

    /// Eye position: the camera sits a `radius` back along the oriented view direction.
    pub fn eye(&self) -> Vec3 {
        self.target - (self.orientation * Vec3::Z) * self.radius
    }

    /// Apply a mouse-drag delta (pixels): yaw about world-up, pitch about the current right axis.
    /// Both are world-space pre-multiplications, so the camera can pass over the poles freely.
    pub fn rotate(&mut self, dx: f32, dy: f32) {
        let sens = 0.005;
        let right = self.orientation * Vec3::X;
        let yaw = Quat::from_axis_angle(Vec3::Y, -dx * sens);
        let pitch = Quat::from_axis_angle(right, -dy * sens);
        self.orientation = (yaw * pitch * self.orientation).normalize();
    }

    /// Pan: translate the look-at target in the view plane (grab-style — the scene follows the
    /// cursor). `dx`/`dy` are pixel deltas; the move is scaled so it tracks 1:1 at the target depth,
    /// so you can shift the pivot off-centre and into the scene (e.g. inside a room).
    pub fn pan(&mut self, dx: f32, dy: f32, viewport_h: f32) {
        let world_per_px = 2.0 * self.radius * (self.fovy * 0.5).tan() / viewport_h.max(1.0);
        // orientation·(-dx,-dy,0) = -dx·(camera right) - dy·(camera up): drag right → scene right.
        self.target += self.orientation * Vec3::new(-dx, -dy, 0.0) * world_per_px;
    }

    /// Set orientation from yaw/pitch angles (radians) — used by the offscreen `--orbit` renders.
    pub fn set_angles(&mut self, yaw: f32, pitch: f32) {
        self.orientation =
            Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
    }

    pub fn camera(&self, width: u32, height: u32) -> Camera {
        let up = self.orientation * Vec3::Y;
        let focal = (height as f32 * 0.5) / (self.fovy * 0.5).tan();
        Camera::look_at(self.eye(), self.target, up, focal, focal, width, height)
    }
}

/// A procedural demo scene: ~1200 colored gaussians on a Fibonacci sphere (color ← direction).
/// Pure-compute and deterministic, so it works identically on native and WASM with no asset file —
/// a denser, orbit-able cloud than `synthetic_scene` for showing the viewer off.
pub fn demo_scene() -> (Vec<Gaussian>, Background) {
    let n: usize = 1200;
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt()); // golden angle
    let mut gaussians = Vec::with_capacity(n);
    for i in 0..n {
        let t = (i as f32 + 0.5) / n as f32;
        let y = 1.0 - 2.0 * t; // -1..1
        let r = (1.0 - y * y).max(0.0).sqrt();
        let phi = i as f32 * golden;
        let pos = Vec3::new(r * phi.cos(), y, r * phi.sin());
        let col = Vec3::new(0.5 + 0.5 * pos.x, 0.5 + 0.5 * pos.y, 0.5 + 0.5 * pos.z);
        gaussians.push(Gaussian {
            mean: pos,
            log_scale: Vec3::splat(0.06f32.ln()),
            quat: [1.0, 0.0, 0.0, 0.0],
            color_dc: (col - Vec3::splat(0.5)) / SH_C0, // invert color_from_dc → color ≈ `col`
            opacity_raw: 4.0,
            sh: None,
        });
    }
    (gaussians, Background { w_b: 0.02, c_b: Vec3::ZERO })
}

/// A tiny hand-made scene for milestone 2/3: three overlapping gaussians on a gray background.
/// Deliberately depth-overlapping so the depth-free WSR averaging (weakness A1) is visible.
pub fn synthetic_scene() -> (Vec<Gaussian>, Background) {
    let g = |mean: Vec3, s: f32, color: Vec3, op: f32| Gaussian {
        mean,
        log_scale: Vec3::splat(s.ln()),
        quat: [1.0, 0.0, 0.0, 0.0],
        color_dc: (color - Vec3::splat(0.5)) / SH_C0, // invert color_from_dc so color≈`color`
        opacity_raw: op,
        sh: None,
    };
    let gaussians = vec![
        g(Vec3::new(-0.3, 0.0, 4.0), 0.30, Vec3::new(0.9, 0.2, 0.2), 4.0),
        g(Vec3::new(0.3, 0.0, 4.2), 0.30, Vec3::new(0.2, 0.9, 0.2), 4.0),
        g(Vec3::new(0.0, 0.25, 3.8), 0.25, Vec3::new(0.2, 0.3, 0.9), 4.0),
    ];
    let bg = Background {
        w_b: 0.05,
        c_b: Vec3::new(0.1, 0.1, 0.12),
    };
    (gaussians, bg)
}
