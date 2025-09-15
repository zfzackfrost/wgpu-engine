struct VertexOut {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec4f,
};

/// @include "struct/VertexBuf"

@vertex
fn vs_main(in: VertexBuf) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4f(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}
struct FragmentOut {
    @location(0) color: vec4f,
};

struct Params {
    tint: vec3f,
};
@group(0) @binding(0)
var<uniform> params: Params;

@fragment
fn fs_main(in: VertexOut) -> FragmentOut {
    var out: FragmentOut;
    let rgb = pow(params.tint * in.color.rgb, vec3f(2.2));
    out.color = vec4f(rgb, in.color.a);
    return out;
}
