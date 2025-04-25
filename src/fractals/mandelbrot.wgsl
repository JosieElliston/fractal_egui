// https://github.com/BenjaminAster/WebGPU-Mandelbrot/blob/main/shader.wgsl
struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) fragmentPosition: vec2<f32>,
}

struct Params {
    // lo_real: f32,
    // lo_imag: f32,
    // hi_real: f32,
    // hi_imag: f32,
    center_real: f32,
    center_imag: f32,
    radius_real: f32,
    radius_imag: f32,
    width: u32,
    height: u32,
    z0_real: f32,
    z0_imag: f32,
    max_depth: u32,
    cycle_depth: u32, // this is a ~sentinel for if it finds a cycle
}

@group(0) @binding(0) var<uniform> params: Params;


fn get_depth(c_real: f32, c_imag: f32) -> u32 {
    var z_real = params.z0_real;
    var z_imag = params.z0_imag;
    var old_real = z_real;
    var old_imag = z_imag;
    var z_real2 = z_real * z_real;
    var z_imag2 = z_imag * z_imag;
    var period_i = 0;
    var period_len = 1;
    for (var depth: u32 = 0; depth < params.max_depth; depth++) {
        if (z_real2 + z_imag2 > 4.0) {
            return depth;
        }
        z_imag = (z_real + z_real) * z_imag + c_imag;
        z_real = z_real2 - z_imag2 + c_real;
        z_real2 = z_real * z_real;
        z_imag2 = z_imag * z_imag;

        if ((old_real == z_real) && (old_imag == z_imag)) {
            // // TODO: remove
            // return depth;
            // return params.cycle_depth;
        }

        period_i += 1;
        if (period_i > period_len) {
            period_i = 0;
            period_len += 1;
            old_real = z_real;
            old_imag = z_imag;
        }
    }
    return params.max_depth;
}

@vertex
fn vertex_main(@builtin(vertex_index) vertexIndex: u32) -> VertexOutput {
    // this is so that i don't need to pass in a vertex buffer
	// var positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
	// 	vec2<f32>(1.0, -1.0),
	// 	vec2<f32>(1.0, 1.0),
	// 	vec2<f32>(-1.0, -1.0),
	// 	vec2<f32>(-1.0, 1.0),
	// );
	var positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
		vec2<f32>(1.0, -1.0),
		vec2<f32>(1.0, 1.0),
		vec2<f32>(-1.0, 1.0),
		vec2<f32>(-1.0, 1.0),
		vec2<f32>(-1.0, -1.0),
		vec2<f32>(1.0, -1.0),
	);
	let position2d: vec2<f32> = positions[vertexIndex];
	return VertexOutput(vec4<f32>(position2d, 0.0, 1.0), position2d);
}

fn get_next_power_of_2(x: u32) -> u32 {
    var i: u32 = 1;
    while (i < x) {
        i = i << 1;
    }
    return i;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let depth = get_depth(
        params.center_real + input.fragmentPosition.x * params.radius_real,
        params.center_imag + input.fragmentPosition.y * params.radius_imag
    );
    var color: f32;
    if depth == params.max_depth {
        color = 0.0;
    } else if depth == params.cycle_depth {
        color = 0.0;
    } else if depth == 0 {
        color = 1.0;
    } else {
        // color = clamp(log(f32(depth)) * 35.0 / 255.0, 0.0, 1.0);
        // color = clamp(0.1 * log(f32(depth)), 0.0, 1.0);
        // TODO: if depth is very small, this might break
        let next_power_of_2 = get_next_power_of_2(depth);
        // if next_power_of_2 < depth {
        //     return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        // }
        let prev_power_of_2 = next_power_of_2 >> 1;
        // if prev_power_of_2 > depth {
        //     return vec4<f32>(0.0, 1.0, 0.0, 1.0);
        // }
        // let depth = f32(depth);
        let t = f32(depth - prev_power_of_2) / f32(next_power_of_2 - prev_power_of_2);
        color = t;
        
    }
    // return vec4<f32>(color, color, color, 1.0);
    return turbo(color, 0.0, 1.0);
}

// Copyright 2019 Google LLC.
// SPDX-License-Identifier: Apache-2.0

// Polynomial approximation in GLSL for the Turbo colormap
// Original LUT: https://gist.github.com/mikhailov-work/ee72ba4191942acecc03fe6da94fc73f

// Authors:
//   Colormap Design: Anton Mikhailov (mikhailov@google.com)
//   GLSL Approximation: Ruofei Du (ruofei@google.com)
//   WGSL Port: Andrew Farkas

fn turbo(value: f32, min: f32, max: f32) -> vec4<f32> {
    let kRedVec4: vec4<f32> = vec4(0.13572138, 4.61539260, -42.66032258, 132.13108234);
    let kGreenVec4: vec4<f32> = vec4(0.09140261, 2.19418839, 4.84296658, -14.18503333);
    let kBlueVec4: vec4<f32> = vec4(0.10667330, 12.64194608, -60.58204836, 110.36276771);
    let kRedVec2: vec2<f32> = vec2(-152.94239396, 59.28637943);
    let kGreenVec2: vec2<f32> = vec2(4.27729857, 2.82956604);
    let kBlueVec2: vec2<f32> = vec2(-89.90310912, 27.34824973);

    let x = saturate((value - min) / (max - min));
    if abs(x) < 0.51 && abs(x) > 0.49 {
        return vec4(1.0, 1.0, 1.0, 1.0);
    }
    let v4: vec4<f32> = vec4( 1.0, x, x * x, x * x * x);
    let v2: vec2<f32> = v4.zw * v4.z;
    return vec4(
        dot(v4, kRedVec4)   + dot(v2, kRedVec2),
        dot(v4, kGreenVec4) + dot(v2, kGreenVec2),
        dot(v4, kBlueVec4)  + dot(v2, kBlueVec2),
        1.0,
    );
}

fn linear_to_gamma(linear: vec4<f32>) -> vec4<f32> {
    // from http://chilliant.blogspot.com/2012/08/srgb-approximations-for-hlsl.html
    return max(1.055 * pow(linear, vec4(0.416666667)) - 0.055, vec4(0.0));
}
