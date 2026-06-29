// Standard 3DGS reference pane (GPU-projected): alpha = sigmoid(opacity)·exp(−Q/2), composited
// back-to-front with painter's-order "over" blending. Mirrors ../tt-splat/spike/arms.render_D.
//
// Like the WSR pane, raw gaussians upload once and the vertex shader does the EWA projection. The
// instance buffer is kept in depth-sorted order (CPU sort of the means each camera move), so drawing
// instances in order resolves occlusion. Color is SH degree-0 (DC) here — the GPU path drops the
// higher-order SH the CPU path evaluated (re-add later via a coefficient buffer if needed).

const SH_C0: f32 = 0.2820947917738781;

struct Cam {
    r_v0: vec4<f32>,
    r_v1: vec4<f32>,
    r_v2: vec4<f32>,
    t_v_near: vec4<f32>, // t_v.xyz, near
    intr: vec4<f32>,     // fx, fy, cx, cy
    misc: vec4<f32>,     // viewport.xy, radius_sigma², blur_eps
};
@group(0) @binding(0) var<uniform> cam: Cam;

struct GaussianIn {
    @location(0) mean_opacity: vec4<f32>,
    @location(1) log_scale: vec4<f32>,
    @location(2) quat: vec4<f32>,
    @location(3) color_dc: vec4<f32>,
};

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) frag_px: vec2<f32>,
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

fn quat_to_mat(q4: vec4<f32>) -> mat3x3<f32> {
    let q = q4 / max(length(q4), 1e-12);
    let w = q.x; let x = q.y; let y = q.z; let z = q.w;
    let r00 = 1.0 - 2.0 * (y * y + z * z); let r01 = 2.0 * (x * y - w * z); let r02 = 2.0 * (x * z + w * y);
    let r10 = 2.0 * (x * y + w * z); let r11 = 1.0 - 2.0 * (x * x + z * z); let r12 = 2.0 * (y * z - w * x);
    let r20 = 2.0 * (x * z - w * y); let r21 = 2.0 * (y * z + w * x); let r22 = 1.0 - 2.0 * (x * x + y * y);
    return mat3x3<f32>(vec3<f32>(r00, r10, r20), vec3<f32>(r01, r11, r21), vec3<f32>(r02, r12, r22));
}

@vertex
fn vs_gs(@builtin(vertex_index) vi: u32, g: GaussianIn) -> VertexOut {
    let r_v = mat3x3<f32>(cam.r_v0.xyz, cam.r_v1.xyz, cam.r_v2.xyz);
    let t_v = cam.t_v_near.xyz;
    let near = cam.t_v_near.w;
    let fx = cam.intr.x; let fy = cam.intr.y; let cx = cam.intr.z; let cy = cam.intr.w;

    let rot = quat_to_mat(g.quat);
    let s = exp(g.log_scale.xyz);
    let s2 = s * s;
    let m = mat3x3<f32>(rot[0] * s2.x, rot[1] * s2.y, rot[2] * s2.z);
    let cov = m * transpose(rot);

    let mu_cam = r_v * g.mean_opacity.xyz + t_v;
    let depth = mu_cam.z;
    let z = max(depth, near);
    let mu2d = vec2<f32>(fx * mu_cam.x / z + cx, fy * mu_cam.y / z + cy);

    let sc = r_v * cov * transpose(r_v);
    let j0 = vec3<f32>(fx / z, 0.0, -fx * mu_cam.x / (z * z));
    let j1 = vec3<f32>(0.0, fy / z, -fy * mu_cam.y / (z * z));
    let blur = cam.misc.w;
    let s00 = dot(j0, sc * j0) + blur;
    let s01 = dot(j0, sc * j1);
    let s11 = dot(j1, sc * j1) + blur;
    let det = max(s00 * s11 - s01 * s01, 1e-12);
    let conic = vec3<f32>(s11 / det, -s01 / det, s00 / det);
    let half = vec2<f32>(sqrt(cam.misc.z * s00), sqrt(cam.misc.z * s11));

    let frag_px = mu2d + corner(vi) * half;
    let ndc = vec2<f32>(2.0 * frag_px.x / cam.misc.x - 1.0, 1.0 - 2.0 * frag_px.y / cam.misc.y);

    var out: VertexOut;
    out.clip = select(vec4<f32>(2.0, 2.0, 2.0, 1.0), vec4<f32>(ndc, 0.0, 1.0), depth > near);
    out.frag_px = frag_px;
    out.mu2d = mu2d;
    out.conic = conic;
    out.opacity = 1.0 / (1.0 + exp(-g.mean_opacity.w));
    out.color = max(vec3<f32>(0.0), vec3<f32>(0.5) + SH_C0 * g.color_dc.xyz);
    return out;
}

@fragment
fn fs_gs(in: VertexOut) -> @location(0) vec4<f32> {
    let d = in.frag_px - in.mu2d;
    let q = in.conic.x * d.x * d.x + 2.0 * in.conic.y * d.x * d.y + in.conic.z * d.y * d.y;
    let alpha = min(0.99, in.opacity * exp(-0.5 * q));
    return vec4<f32>(in.color, alpha);
}
