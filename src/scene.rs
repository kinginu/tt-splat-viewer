//! CPU-side scene + geometry preprocess. Mirrors `../tt-splat/spike/{geometry,camera,forward}.py`.
//!
//! The per-gaussian "G-setup" (quat→rotation, 3D cov, EWA projection to a 2D mean + conic) is cheap
//! and done on the CPU; the per-pixel hot path (Q, poly-splat, WSR blend) lives in `shader.wgsl`.
//! See `CLAUDE.md` §2. Numbers here must match the oracle — verify via the PSNR harness (§5), not by eye.

use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Vec3};

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
    pub _pad: f32,
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

/// Project every gaussian to an `InstanceRaw`; culled gaussians (`keep == false`) are dropped.
/// Mirrors `geometry.project_ewa` + `forward.color_from_dc` + the opacity sigmoid.
pub fn preprocess(gaussians: &[Gaussian], cam: &Camera) -> Vec<InstanceRaw> {
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

        // Footprint half-extent: the ellipse {dᵀ M d ≤ k} is axis-bounded by sqrt(k·Sigma2d_ii).
        let half_extent = [(K * s00).sqrt(), (K * s11).sqrt()];

        let opacity = 1.0 / (1.0 + (-g.opacity_raw).exp());
        let color = [
            (0.5 + SH_C0 * g.color_dc.x).max(0.0),
            (0.5 + SH_C0 * g.color_dc.y).max(0.0),
            (0.5 + SH_C0 * g.color_dc.z).max(0.0),
        ];

        out.push(InstanceRaw {
            mu2d,
            half_extent,
            conic,
            opacity,
            color,
            _pad: 0.0,
        });
    }
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

/// Load a standard INRIA-3DGS `.ply` (binary-little-endian or ascii). Reads vertex properties by
/// name: `x y z`, `scale_0..2` (log), `rot_0..3` (quaternion `w,x,y,z`), `f_dc_0..2` (SH-deg0 color),
/// `opacity` (pre-sigmoid). Any extra properties (normals, `f_rest_*`) are skipped — WSR uses DC only.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_ply(path: &std::path::Path) -> std::io::Result<Vec<Gaussian>> {
    use std::io::{Error, ErrorKind};
    let bytes = std::fs::read(path)?;

    let marker = b"end_header\n";
    let header_end = bytes
        .windows(marker.len())
        .position(|w| w == marker)
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "no end_header"))?;
    let data_start = header_end + marker.len();
    let header = String::from_utf8_lossy(&bytes[..header_end]);

    let mut is_ascii = false;
    let mut count = 0usize;
    // (name, byte size) in file order, for the `vertex` element.
    let mut props: Vec<(String, usize)> = Vec::new();
    let type_size = |t: &str| -> usize {
        match t {
            "char" | "uchar" | "int8" | "uint8" => 1,
            "short" | "ushort" | "int16" | "uint16" => 2,
            "int" | "uint" | "int32" | "uint32" | "float" | "float32" => 4,
            "double" | "float64" => 8,
            _ => 4,
        }
    };
    for line in header.lines() {
        let tok: Vec<&str> = line.split_whitespace().collect();
        match tok.as_slice() {
            ["format", fmt, ..] => is_ascii = fmt.starts_with("ascii"),
            ["element", "vertex", n] => count = n.parse().unwrap_or(0),
            ["property", ty, name] => props.push((name.to_string(), type_size(ty))),
            _ => {}
        }
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

    let g_at = |vals: &dyn Fn(&str) -> f32| Gaussian {
        mean: Vec3::new(vals("x"), vals("y"), vals("z")),
        log_scale: Vec3::new(vals("scale_0"), vals("scale_1"), vals("scale_2")),
        quat: [vals("rot_0"), vals("rot_1"), vals("rot_2"), vals("rot_3")],
        color_dc: Vec3::new(vals("f_dc_0"), vals("f_dc_1"), vals("f_dc_2")),
        opacity_raw: vals("opacity"),
    };

    let mut out = Vec::with_capacity(count);
    if is_ascii {
        let text = String::from_utf8_lossy(&bytes[data_start..]);
        for line in text.lines().filter(|l| !l.trim().is_empty()).take(count) {
            let nums: Vec<f32> = line.split_whitespace().filter_map(|t| t.parse().ok()).collect();
            let vals = |name: &str| nums[col[name]];
            out.push(g_at(&vals));
        }
    } else {
        for i in 0..count {
            let base = data_start + i * stride;
            if base + stride > bytes.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "ply body truncated"));
            }
            let vals = |name: &str| -> f32 {
                let o = base + offset[col[name]];
                f32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]])
            };
            out.push(g_at(&vals));
        }
    }
    Ok(out)
}

/// An orbit camera: rotate `yaw`/`pitch` around `target` at `radius`, vertical fov `fovy` (radians).
#[derive(Clone, Copy, Debug)]
pub struct Orbit {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub radius: f32,
    pub fovy: f32,
}

impl Orbit {
    /// Frame a gaussian set: centroid as target, radius from the bounding sphere + fov.
    pub fn frame(gaussians: &[Gaussian]) -> Self {
        let fovy = 60f32.to_radians();
        if gaussians.is_empty() {
            return Orbit { target: Vec3::ZERO, yaw: 0.0, pitch: 0.0, radius: 5.0, fovy };
        }
        let centroid = gaussians.iter().map(|g| g.mean).sum::<Vec3>() / gaussians.len() as f32;
        let bound = gaussians
            .iter()
            .map(|g| (g.mean - centroid).length())
            .fold(0.0f32, f32::max)
            .max(1e-3);
        // distance so the bounding sphere fits the vertical fov, with margin.
        let radius = 1.4 * bound / (fovy * 0.5).sin();
        Orbit { target: centroid, yaw: 0.0, pitch: 0.0, radius, fovy }
    }

    /// Eye position for the current orbit angles (OpenCV: looking toward +forward, +y down handled by `look_at`).
    pub fn eye(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let dir = Vec3::new(cp * sy, sp, cp * cy);
        self.target - dir * self.radius
    }

    pub fn camera(&self, width: u32, height: u32) -> Camera {
        let focal = (height as f32 * 0.5) / (self.fovy * 0.5).tan();
        Camera::look_at(self.eye(), self.target, Vec3::new(0.0, 1.0, 0.0), focal, focal, width, height)
    }
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
