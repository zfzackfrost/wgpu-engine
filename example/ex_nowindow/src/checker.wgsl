
// Bindings:
// 0: storage texture (rgba8unorm)

@group(0) @binding(0)
var outImage : texture_storage_2d<rgba8unorm, write>;

struct ComputeInputs {
    @builtin(global_invocation_id) gid : vec3<u32>,
};

fn doWork(inputs: ComputeInputs) {
    // Checker size (pixels per square)
    let checker_size : u32 = 32u;

    // Compute which checker cell this pixel belongs to
    let cx : u32 = inputs.gid.x / checker_size;
    let cy : u32 = inputs.gid.y / checker_size;

    // Alternate color based on parity
    let is_dark : bool = (cx + cy) % 2u == 0u;

    let dark_color : vec4<f32> = vec4<f32>(0.1, 0.1, 0.1, 1.0);
    let light_color : vec4<f32> = vec4<f32>(0.9, 0.9, 0.9, 1.0);

    // Use select(a, b, condition): returns `a` if condition is false, else `b`
    let color : vec4<f32> = select(light_color, dark_color, is_dark);

    textureStore(outImage, vec2<i32>(inputs.gid.xy), color);
}

@compute @workgroup_size(8, 8)
fn cs_main(inputs: ComputeInputs) {
    let size : vec2<u32> = textureDimensions(outImage);
    if (inputs.gid.x < size.x && inputs.gid.y < size.y) {
        doWork(inputs);
    }
}

