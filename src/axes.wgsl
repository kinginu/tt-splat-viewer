// XYZ axis gizmo: colored line segments (X=red, Y=green, Z=blue) whose endpoints are projected to
// NDC on the CPU (scene::Camera::project_ndc), so this shader just passes them through.

struct VertexIn {
    @location(0) pos: vec2<f32>,   // clip-space NDC
    @location(1) color: vec3<f32>,
};

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_axes(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip = vec4<f32>(in.pos, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_axes(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
