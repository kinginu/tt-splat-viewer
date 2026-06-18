// tt-splat poly-splat + Weighted Sum Rendering (WSR) — the per-pixel hot path.
// Mirrors ../tt-splat/spike/{forward,arms}.py. See CLAUDE.md §2.
//
// Pass 1 (splat): each gaussian draws one additive quad into an Rgba16Float accumulator:
//     RGB += w·color,   A += w,   with  w = sigmoid(opacity)·max(0, 1 - Q/k)²
// Because N = Σ w·color and D = Σ w are commutative sums, additive blending == WSR. No sort.
// Pass 2 (composite): one fullscreen triangle divides:  C = (RGB + w_b·c_b) / (A + w_b).

const K: f32 = 4.0; // poly-splat width (spike uses 4.0)

// ---------- Pass 1: splat ----------

struct Globals {
    viewport: vec2<f32>, // (width, height) in pixels
};
@group(0) @binding(0) var<uniform> globals: Globals;

struct InstanceIn {
    @location(0) mu2d: vec2<f32>,
    @location(1) half_extent: vec2<f32>,
    @location(2) conic: vec3<f32>,   // (a, b, c) = inverse 2D covariance
    @location(3) opacity: f32,       // o = sigmoid(opacity_raw)
    @location(4) color: vec3<f32>,
};

struct SplatVertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) frag_px: vec2<f32>, // pixel-space position of this fragment
    @location(1) mu2d: vec2<f32>,
    @location(2) conic: vec3<f32>,
    @location(3) opacity: f32,
    @location(4) color: vec3<f32>,
};

// Quad corners as a triangle-strip (4 verts), in {-1,+1}².
fn corner(vi: u32) -> vec2<f32> {
    switch vi {
        case 0u: { return vec2<f32>(-1.0, -1.0); }
        case 1u: { return vec2<f32>( 1.0, -1.0); }
        case 2u: { return vec2<f32>(-1.0,  1.0); }
        default: { return vec2<f32>( 1.0,  1.0); }
    }
}

@vertex
fn vs_splat(@builtin(vertex_index) vi: u32, inst: InstanceIn) -> SplatVertexOut {
    let offset = corner(vi) * inst.half_extent;
    let frag_px = inst.mu2d + offset;

    // pixel coords (x right, y down) -> clip space (x right, y up)
    let ndc = vec2<f32>(
        2.0 * frag_px.x / globals.viewport.x - 1.0,
        1.0 - 2.0 * frag_px.y / globals.viewport.y,
    );

    var out: SplatVertexOut;
    out.clip = vec4<f32>(ndc, 0.0, 1.0);
    out.frag_px = frag_px;
    out.mu2d = inst.mu2d;
    out.conic = inst.conic;
    out.opacity = inst.opacity;
    out.color = inst.color;
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
