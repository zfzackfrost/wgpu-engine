struct ComputeInputs {
    @builtin(global_invocation_id) gid: vec3<u32>,
};

struct Params {
    color_a: vec4f,
    color_b: vec4f,
    checker_size: u32,
};

// Binding Group 0:
// 0: storage texture (rgba8unorm)
// 1: uniform buffer (Params)

@group(0) @binding(0)
var out_image: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> params: Params;


fn store_checker_tex(inputs: ComputeInputs) {
    // Compute which checker cell this pixel belongs to
    let cx: u32 = inputs.gid.x / params.checker_size;
    let cy: u32 = inputs.gid.y / params.checker_size;

    // Alternate color based on parity
    let is_dark: bool = (cx + cy) % 2u == 0u;
    let color: vec4<f32> = select(params.color_a, params.color_b, is_dark);

    // Store `color` in `out_image`
    textureStore(out_image, vec2<i32>(inputs.gid.xy), color);
}

@compute @workgroup_size(8, 8)
fn cs_main(inputs: ComputeInputs) {
    let size: vec2<u32> = textureDimensions(out_image);
    if (inputs.gid.x < size.x && inputs.gid.y < size.y) {
        store_checker_tex(inputs);
    }
}

