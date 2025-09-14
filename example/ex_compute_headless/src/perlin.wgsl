// Generates 2D Perlin noise using a compute shader

@group(0) @binding(0) var<storage, read_write> output: array<f32>;

struct NoiseParams {
    width: u32,
    height: u32,
    scale: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
    offset_x: f32,
    offset_y: f32,
}

@group(0) @binding(1) var<uniform> params: NoiseParams;

// Permutation table for Perlin noise (simplified version)
const PERM: array<u32, 256> = array<u32, 256>(
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225,
    140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148,
    247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32,
    57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122,
    60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54,
    65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169,
    200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3, 64,
    52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212,
    207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213,
    119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9,
    129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104,
    218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162, 241,
    81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157,
    184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93,
    222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180
);

// Hash function for permutation
fn hash(x: u32) -> u32 {
    return PERM[x & 255u];
}

// Fade function (smoothstep)
fn fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

// Gradient function - returns dot product of gradient vector and distance vector
fn grad(hash_val: u32, x: f32, y: f32) -> f32 {
    let h = hash_val & 3u;
    let u = select(y, x, h < 2u);
    let v = select(x, y, h < 2u);
    let u_sign = select(u, -u, (h & 1u) != 0u);
    let v_sign = select(v, -v, (h & 2u) != 0u);
    return u_sign + v_sign;
}

// 2D Perlin noise function
fn noise2d(x: f32, y: f32) -> f32 {
    // Find unit square that contains point
    let xi = u32(floor(x)) & 255u;
    let yi = u32(floor(y)) & 255u;
    
    // Find relative x, y of point in square
    let xf = x - floor(x);
    let yf = y - floor(y);
    
    // Compute fade curves for x and y
    let u = fade(xf);
    let v = fade(yf);
    
    // Hash coordinates of 4 square corners
    let aa = hash(hash(xi) + yi);
    let ab = hash(hash(xi) + yi + 1u);
    let ba = hash(hash(xi + 1u) + yi);
    let bb = hash(hash(xi + 1u) + yi + 1u);
    
    // Calculate gradients at each corner
    let grad_aa = grad(aa, xf, yf);
    let grad_ab = grad(ab, xf, yf - 1.0);
    let grad_ba = grad(ba, xf - 1.0, yf);
    let grad_bb = grad(bb, xf - 1.0, yf - 1.0);
    
    // Interpolate the results
    let x1 = mix(grad_aa, grad_ba, u);
    let x2 = mix(grad_ab, grad_bb, u);
    
    return mix(x1, x2, v);
}

// Fractal Brownian Motion (fBm) - combines multiple octaves of noise
fn fbm(x: f32, y: f32, octaves: u32, persistence: f32, lacunarity: f32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = 1.0;
    var max_value = 0.0;
    for (var i = 0u; i < octaves; i++) {
        value += noise2d(x * frequency, y * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    return value / max_value;
}

fn gen_noise(x: u32, y: u32) {
    // Calculate normalized coordinates
    let norm_x = (f32(x) + params.offset_x) * params.scale / f32(params.width);
    let norm_y = (f32(y) + params.offset_y) * params.scale / f32(params.height);
    
    // Generate noise value using fBm
    let noise_value = fbm(norm_x, norm_y, params.octaves, params.persistence, params.lacunarity);
    
    // Normalize to [0, 1] range
    let normalized_noise = (noise_value + 1.0) * 0.5;
    
    // Store in output buffer
    let index = y * params.width + x;
    output[index] = normalized_noise;
}

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Bounds checking
    if (x < params.width && y < params.height) {
        gen_noise(x, y);
    }
}
