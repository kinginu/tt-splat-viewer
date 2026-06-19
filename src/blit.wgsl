// Side-by-side composite: paint pane A (left) and pane B (right) into the surface, with a 2px divider.
// Each pane texture is 1:1 with its half of the surface, so textureLoad by integer pixel is exact.

@group(0) @binding(0) var tex_a: texture_2d<f32>;
@group(0) @binding(1) var tex_b: texture_2d<f32>;

struct Split {
    left_w: u32,  // width of the left pane in pixels; right pane starts here
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};
@group(0) @binding(2) var<uniform> split: Split;

@vertex
fn vs_blit(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_blit(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let x = i32(pos.x);
    let y = i32(pos.y);
    let lw = i32(split.left_w);
    if (x == lw - 1 || x == lw) {
        return vec4<f32>(0.16, 0.17, 0.22, 1.0); // divider line
    }
    if (x < lw) {
        return textureLoad(tex_a, vec2<i32>(x, y), 0);
    }
    return textureLoad(tex_b, vec2<i32>(x - lw, y), 0);
}
