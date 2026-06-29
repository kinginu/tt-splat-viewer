// Side-by-side composite: pane A (left) and pane B (right) into the surface, with a 2px divider.
// The pane targets may be rendered at a lower resolution (dynamic-resolution during interaction), so
// this samples them (linear) by normalized UV rather than 1:1 textureLoad — upscaling as needed.

@group(0) @binding(0) var tex_a: texture_2d<f32>;
@group(0) @binding(1) var tex_b: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

struct Split {
    left_w: u32,  // left pane width in surface pixels; right pane starts here
    right_w: u32, // right pane width
    height: u32,  // surface height
    _pad: u32,
};
@group(0) @binding(3) var<uniform> split: Split;

@vertex
fn vs_blit(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_blit(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let lw = f32(split.left_w);
    let rw = f32(split.right_w);
    let h = f32(split.height);
    let xi = i32(pos.x);
    if (xi == i32(split.left_w) - 1 || xi == i32(split.left_w)) {
        return vec4<f32>(0.16, 0.17, 0.22, 1.0); // divider line
    }
    if (pos.x < lw) {
        return textureSampleLevel(tex_a, samp, vec2<f32>(pos.x / lw, pos.y / h), 0.0);
    }
    return textureSampleLevel(tex_b, samp, vec2<f32>((pos.x - lw) / rw, pos.y / h), 0.0);
}
