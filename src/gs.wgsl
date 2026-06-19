// Standard 3DGS reference renderer (the "original method" pane): alpha = sigmoid(opacity)·exp(−Q/2),
// composited back-to-front with painter's-order "over" blending. Mirrors ../tt-splat/spike/arms.render_D
// (minus the SH view-dependence — colors here are SH degree-0 / DC only for now).
//
// Instances arrive already depth-sorted (far → near) from scene::preprocess_sorted; this pass just
// draws them in order with "over" blending, so occlusion is resolved correctly (unlike WSR).

struct Globals {
    viewport: vec2<f32>, // (width, height) of this pane in pixels
};
@group(0) @binding(0) var<uniform> globals: Globals;

struct InstanceIn {
    @location(0) mu2d: vec2<f32>,
    @location(1) half_extent: vec2<f32>,
    @location(2) conic: vec3<f32>, // (a, b, c) = inverse 2D covariance
    @location(3) opacity: f32,     // o = sigmoid(opacity_raw)
    @location(4) color: vec3<f32>,
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

@vertex
fn vs_gs(@builtin(vertex_index) vi: u32, inst: InstanceIn) -> VertexOut {
    let frag_px = inst.mu2d + corner(vi) * inst.half_extent;
    let ndc = vec2<f32>(
        2.0 * frag_px.x / globals.viewport.x - 1.0,
        1.0 - 2.0 * frag_px.y / globals.viewport.y,
    );
    var out: VertexOut;
    out.clip = vec4<f32>(ndc, 0.0, 1.0);
    out.frag_px = frag_px;
    out.mu2d = inst.mu2d;
    out.conic = inst.conic;
    out.opacity = inst.opacity;
    out.color = inst.color;
    return out;
}

@fragment
fn fs_gs(in: VertexOut) -> @location(0) vec4<f32> {
    let d = in.frag_px - in.mu2d;
    let q = in.conic.x * d.x * d.x + 2.0 * in.conic.y * d.x * d.y + in.conic.z * d.y * d.y;
    let alpha = min(0.99, in.opacity * exp(-0.5 * q)); // transcendental: fine, this is the reference pane
    // Straight (non-premultiplied) color + alpha; the pipeline's "over" blend does the compositing.
    return vec4<f32>(in.color, alpha);
}
