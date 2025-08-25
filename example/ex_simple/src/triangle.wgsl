struct VertexOut {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
};
struct VertexIn {
    @builtin(vertex_index) index: u32,
};
var<private> VERTICES: array<vec2f, 3> = array<vec2f, 3>(
    vec2f(0.0, 0.5),   // Top-center
    vec2f(-0.5, -0.5), // Bottom-left
    vec2f(0.5, -0.5),  // Bottom-right
);
var<private> COLORS: array<vec3f, 3> = array<vec3f, 3>(
    vec3f(1.0, 1.0, 0.0), // Top-center
    vec3f(1.0, 0.0, 0.0), // Bottom-left
    vec3f(0.0, 0.0, 1.0), // Bottom-right
);

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4f(VERTICES[in.index], 0.0, 1.0);
    out.color = COLORS[in.index];
    return out;
}
struct FragmentOut {
    @location(0) color: vec4f,
};

@fragment
fn fs_main(in: VertexOut) -> FragmentOut {
    var out: FragmentOut;
    out.color = vec4f(in.color, 1.0);
    return out;
}
