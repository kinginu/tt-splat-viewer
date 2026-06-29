// tt-splat poly-splat + Weighted Sum Rendering (WSR) — the per-pixel hot path.
// Mirrors ../tt-splat/spike/{forward,arms}.py. See CLAUDE.md §2.
//
// Pass 1 (splat): each gaussian draws one additive quad into an Rgba16Float accumulator:
//     RGB += w·color,   A += w,   with  w = sigmoid(opacity)·max(0, 1 - Q/k)²
// Because N = Σ w·color and D = Σ w are commutative sums, additive blending == WSR. No sort.
// Pass 2 (composite): one fullscreen triangle divides:  C = (RGB + w_b·c_b) / (A + w_b).

const K: f32 = 4.0;          // poly-splat width (spike uses 4.0)
const SH_C0: f32 = 0.2820947917738781;

// ---------- Pass 1: splat (GPU projection) ----------
//
// The gaussians are uploaded once as raw instance data; this vertex shader does the EWA projection
// itself (mirrors scene::preprocess), so a camera move only updates the small `cam` uniform — no
// per-frame CPU work, no instance re-upload.

struct Cam {
    r_v0: vec4<f32>,     // columns of R_v (world->cam rotation)
    r_v1: vec4<f32>,
    r_v2: vec4<f32>,
    t_v_near: vec4<f32>, // t_v.xyz, near
    intr: vec4<f32>,     // fx, fy, cx, cy
    misc: vec4<f32>,     // viewport.xy, radius_sigma², blur_eps
};
@group(0) @binding(0) var<uniform> cam: Cam;

struct GaussianIn {
    @location(0) mean_opacity: vec4<f32>, // mean.xyz, opacity_raw
    @location(1) log_scale: vec4<f32>,    // log_scale.xyz, _
    @location(2) quat: vec4<f32>,         // w, x, y, z
    @location(3) color_dc: vec4<f32>,     // color_dc.xyz, _
};

struct SplatVertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) frag_px: vec2<f32>, // pixel-space position of this fragment
    @location(1) mu2d: vec2<f32>,
    @location(2) conic: vec3<f32>,
    @location(3) opacity: f32,
    @location(4) color: vec3<f32>,
};

fn corner(vi: u32) -> vec2<f32> {
    switch vi {
        case 0u: { return vec2<f32>(-1.0, -1.0); }
        case 1u: { return vec2<f32>( 1.0, -1.0); }
        case 2u: { return vec2<f32>(-1.0,  1.0); }
        default: { return vec2<f32>( 1.0,  1.0); }
    }
}

// quat [w,x,y,z] -> rotation matrix (columns), matching geometry.quat_to_rotmat.
fn quat_to_mat(q4: vec4<f32>) -> mat3x3<f32> {
    let q = q4 / max(length(q4), 1e-12);
    let w = q.x; let x = q.y; let y = q.z; let z = q.w;
    let r00 = 1.0 - 2.0 * (y * y + z * z); let r01 = 2.0 * (x * y - w * z); let r02 = 2.0 * (x * z + w * y);
    let r10 = 2.0 * (x * y + w * z); let r11 = 1.0 - 2.0 * (x * x + z * z); let r12 = 2.0 * (y * z - w * x);
    let r20 = 2.0 * (x * z - w * y); let r21 = 2.0 * (y * z + w * x); let r22 = 1.0 - 2.0 * (x * x + y * y);
    return mat3x3<f32>(vec3<f32>(r00, r10, r20), vec3<f32>(r01, r11, r21), vec3<f32>(r02, r12, r22));
}

// Compute the per-gaussian splat parameters (mu2d, conic, half_extent, color, opacity, keep) — the
// per-vertex part shared by the WSR and 3DGS panes. Mirrors scene::preprocess exactly.
struct Splat {
    mu2d: vec2<f32>,
    conic: vec3<f32>,
    half: vec2<f32>,
    color: vec3<f32>,
    opacity: f32,
    keep: bool,
};
fn project(g: GaussianIn) -> Splat {
    let r_v = mat3x3<f32>(cam.r_v0.xyz, cam.r_v1.xyz, cam.r_v2.xyz);
    let t_v = cam.t_v_near.xyz;
    let near = cam.t_v_near.w;
    let fx = cam.intr.x; let fy = cam.intr.y; let cx = cam.intr.z; let cy = cam.intr.w;

    let rot = quat_to_mat(g.quat);
    let s = exp(g.log_scale.xyz);
    let s2 = s * s;
    let m = mat3x3<f32>(rot[0] * s2.x, rot[1] * s2.y, rot[2] * s2.z); // R·diag(s²)
    let cov = m * transpose(rot);                                     // Σ3 = R·diag(s²)·Rᵀ

    let mu_cam = r_v * g.mean_opacity.xyz + t_v;
    let depth = mu_cam.z;
    let z = max(depth, near);
    let mu2d = vec2<f32>(fx * mu_cam.x / z + cx, fy * mu_cam.y / z + cy);

    let sc = r_v * cov * transpose(r_v);                              // Σ_cam
    let j0 = vec3<f32>(fx / z, 0.0, -fx * mu_cam.x / (z * z));        // EWA Jacobian rows
    let j1 = vec3<f32>(0.0, fy / z, -fy * mu_cam.y / (z * z));
    let blur = cam.misc.w;
    let s00 = dot(j0, sc * j0) + blur;
    let s01 = dot(j0, sc * j1);
    let s11 = dot(j1, sc * j1) + blur;
    let det = max(s00 * s11 - s01 * s01, 1e-12);

    var o: Splat;
    o.mu2d = mu2d;
    o.conic = vec3<f32>(s11 / det, -s01 / det, s00 / det);
    o.half = vec2<f32>(sqrt(cam.misc.z * s00), sqrt(cam.misc.z * s11));
    o.color = max(vec3<f32>(0.0), vec3<f32>(0.5) + SH_C0 * g.color_dc.xyz);
    o.opacity = 1.0 / (1.0 + exp(-g.mean_opacity.w));
    o.keep = depth > near;
    return o;
}

@vertex
fn vs_splat(@builtin(vertex_index) vi: u32, g: GaussianIn) -> SplatVertexOut {
    let sp = project(g);
    let frag_px = sp.mu2d + corner(vi) * sp.half;
    let ndc = vec2<f32>(
        2.0 * frag_px.x / cam.misc.x - 1.0,
        1.0 - 2.0 * frag_px.y / cam.misc.y,
    );

    var out: SplatVertexOut;
    // Cull (depth ≤ near): collapse the quad off-screen so it produces no fragments.
    out.clip = select(vec4<f32>(2.0, 2.0, 2.0, 1.0), vec4<f32>(ndc, 0.0, 1.0), sp.keep);
    out.frag_px = frag_px;
    out.mu2d = sp.mu2d;
    out.conic = sp.conic;
    out.opacity = sp.opacity;
    out.color = sp.color;
    return out;
}

@fragment
fn fs_splat(in: SplatVertexOut) -> @location(0) vec4<f32> {
    let d = in.frag_px - in.mu2d;
    let a = in.conic.x;
    let b = in.conic.y;
    let c = in.conic.z;
    // Q = a·dx² + 2·b·dx·dy + c·dy²   (forward.quad_form)
    let q = a * d.x * d.x + 2.0 * b * d.x * d.y + c * d.y * d.y;
    // w_geo = max(0, 1 - Q/k)²        (forward.poly_splat_wgeo)
    let u = max(0.0, 1.0 - q / K);
    let w_geo = u * u;
    let w = in.opacity * w_geo;
    // Accumulate premultiplied (Σ w·color, Σ w) via additive blending.
    return vec4<f32>(w * in.color, w);
}

// ---------- Pass 2: composite (WSR divide) ----------

struct Composite {
    c_b: vec3<f32>,
    w_b: f32,
};
@group(0) @binding(0) var accum_tex: texture_2d<f32>;
@group(0) @binding(1) var<uniform> comp: Composite;

@vertex
fn vs_composite(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    // Fullscreen triangle.
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_composite(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    // pos.xy is the pixel center; the accumulator is 1:1 with the target, so floor → texel.
    let coord = vec2<i32>(pos.xy);
    let acc = textureLoad(accum_tex, coord, 0); // (Σ w·color, Σ w)
    let num = acc.rgb + comp.w_b * comp.c_b;
    let den = acc.a + comp.w_b;
    return vec4<f32>(num / max(den, 1e-8), 1.0);
}
