struct VertexOut {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec4f,
};
struct VertexIn {
    @location(0) positon: vec2f,
    @location(1) tex_coords: vec2f,
    @location(2) color: vec4f,
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4f(in.positon, 0.0, 1.0);
    out.color = in.color;
    return out;
}
struct FragmentOut {
    @location(0) color: vec4f,
};

@fragment
fn fs_main(in: VertexOut) -> FragmentOut {
    var out: FragmentOut;
    out.color = in.color;
    return out;
}
