// 各种视觉效果着色器

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct EffectUniforms {
    time: f32,
    intensity: f32,
    param1: f32,
    param2: f32,
    screen_size: vec2<f32>,
    effect_center: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: EffectUniforms;

@group(0) @binding(1)
var input_texture: texture_2d<f32>;

@group(0) @binding(2)
var input_sampler: sampler;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

// 噪声函数
fn random(uv: vec2<f32>) -> f32 {
    return fract(sin(dot(uv, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn noise(uv: vec2<f32>) -> f32 {
    let i = floor(uv);
    let f = fract(uv);
    
    let a = random(i);
    let b = random(i + vec2<f32>(1.0, 0.0));
    let c = random(i + vec2<f32>(0.0, 1.0));
    let d = random(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

// 暗角效果
@fragment
fn fs_vignette(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    let center = vec2<f32>(0.5, 0.5);
    let distance = length(in.uv - center);
    
    let vignette_strength = uniforms.intensity;
    let vignette_smoothness = uniforms.param1;
    let vignette = smoothstep(0.5, 0.5 - vignette_smoothness, distance * vignette_strength);
    
    return vec4<f32>(color.rgb * vignette, color.a);
}

// 色差效果
@fragment
fn fs_chromatic_aberration(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let offset = (in.uv - center) * uniforms.intensity;
    
    let r = textureSample(input_texture, input_sampler, in.uv + offset).r;
    let g = textureSample(input_texture, input_sampler, in.uv).g;
    let b = textureSample(input_texture, input_sampler, in.uv - offset).b;
    let a = textureSample(input_texture, input_sampler, in.uv).a;
    
    return vec4<f32>(r, g, b, a);
}

// 胶片颗粒效果
@fragment
fn fs_film_grain(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    
    let grain_strength = uniforms.intensity;
    let grain_size = uniforms.param1;
    
    let grain_uv = in.uv * uniforms.screen_size / grain_size;
    let grain = noise(grain_uv + uniforms.time) * 2.0 - 1.0;
    
    let luminance = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let grain_amount = grain * grain_strength * (1.0 - luminance * 0.5);
    
    return vec4<f32>(color.rgb + grain_amount, color.a);
}

// 镜头光晕效果
@fragment
fn fs_lens_flare(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    
    let flare_center = uniforms.effect_center;
    let distance_to_center = length(in.uv - flare_center);
    
    // 计算多个光晕元素
    var flare_color = vec3<f32>(0.0);
    
    // 主光晕
    let main_flare = exp(-distance_to_center * 20.0) * uniforms.intensity;
    flare_color += vec3<f32>(1.0, 0.8, 0.6) * main_flare;
    
    // 次级光晕
    let ghost_distance = distance(in.uv, mix(flare_center, vec2<f32>(0.5, 0.5), 0.5));
    let ghost_flare = exp(-ghost_distance * 15.0) * uniforms.intensity * 0.3;
    flare_color += vec3<f32>(0.6, 1.0, 0.8) * ghost_flare;
    
    // 光圈效果
    let ring_distance = abs(distance_to_center - 0.3);
    let ring_flare = exp(-ring_distance * 50.0) * uniforms.intensity * 0.2;
    flare_color += vec3<f32>(1.0, 0.9, 0.7) * ring_flare;
    
    return vec4<f32>(color.rgb + flare_color, color.a);
}

// 屏幕扭曲效果
@fragment
fn fs_screen_distortion(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let distortion_strength = uniforms.intensity;
    
    let distance_to_center = length(in.uv - center);
    let distortion_factor = 1.0 + distortion_strength * distance_to_center * distance_to_center;
    
    let distorted_uv = center + (in.uv - center) * distortion_factor;
    
    if (distorted_uv.x < 0.0 || distorted_uv.x > 1.0 || distorted_uv.y < 0.0 || distorted_uv.y > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    
    return textureSample(input_texture, input_sampler, distorted_uv);
}

// 鱼眼效果
@fragment
fn fs_fisheye(in: VertexOutput) -> @location(0) vec4<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let radius = length(in.uv - center);
    
    if (radius > 0.5) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    
    let theta = atan2(in.uv.y - center.y, in.uv.x - center.x);
    let fisheye_radius = radius * uniforms.intensity;
    let distorted_radius = sin(fisheye_radius * 3.14159 * 0.5);
    
    let distorted_uv = center + vec2<f32>(cos(theta), sin(theta)) * distorted_radius;
    
    return textureSample(input_texture, input_sampler, distorted_uv);
}

// 像素化效果
@fragment
fn fs_pixelate(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_size = uniforms.param1;
    let pixelated_uv = floor(in.uv * uniforms.screen_size / pixel_size) * pixel_size / uniforms.screen_size;
    
    return textureSample(input_texture, input_sampler, pixelated_uv);
}

// 边缘检测效果
@fragment
fn fs_edge_detection(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / uniforms.screen_size;
    
    // Sobel算子
    let sobel_x = mat3x3<f32>(
        vec3<f32>(-1.0, 0.0, 1.0),
        vec3<f32>(-2.0, 0.0, 2.0),
        vec3<f32>(-1.0, 0.0, 1.0)
    );
    
    let sobel_y = mat3x3<f32>(
        vec3<f32>(-1.0, -2.0, -1.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(1.0, 2.0, 1.0)
    );
    
    var gx = 0.0;
    var gy = 0.0;
    
    for (var i = -1; i <= 1; i++) {
        for (var j = -1; j <= 1; j++) {
            let offset = vec2<f32>(f32(i), f32(j)) * texel_size;
            let sample_color = textureSample(input_texture, input_sampler, in.uv + offset);
            let luminance = dot(sample_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
            
            gx += luminance * sobel_x[i + 1][j + 1];
            gy += luminance * sobel_y[i + 1][j + 1];
        }
    }
    
    let edge_strength = sqrt(gx * gx + gy * gy) * uniforms.intensity;
    let edge_color = vec3<f32>(edge_strength);
    
    return vec4<f32>(edge_color, 1.0);
}

// 浮雕效果
@fragment
fn fs_emboss(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / uniforms.screen_size;
    
    // 浮雕卷积核
    let emboss_kernel = mat3x3<f32>(
        vec3<f32>(-2.0, -1.0, 0.0),
        vec3<f32>(-1.0, 1.0, 1.0),
        vec3<f32>(0.0, 1.0, 2.0)
    );
    
    var result = vec3<f32>(0.0);
    
    for (var i = -1; i <= 1; i++) {
        for (var j = -1; j <= 1; j++) {
            let offset = vec2<f32>(f32(i), f32(j)) * texel_size;
            let sample_color = textureSample(input_texture, input_sampler, in.uv + offset);
            result += sample_color.rgb * emboss_kernel[i + 1][j + 1];
        }
    }
    
    result = result * uniforms.intensity + 0.5;
    
    return vec4<f32>(clamp(result, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}

// 旧电视效果
@fragment
fn fs_old_tv(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(input_texture, input_sampler, in.uv);
    
    // 扫描线
    let scanline = sin(in.uv.y * uniforms.screen_size.y * 2.0) * 0.1 + 0.9;
    color.rgb *= scanline;
    
    // 噪声
    let noise_strength = 0.1;
    let tv_noise = random(in.uv + uniforms.time) * noise_strength;
    color.rgb += tv_noise;
    
    // 颜色偏移
    let shift = sin(uniforms.time * 10.0 + in.uv.y * 50.0) * 0.001;
    color.r = textureSample(input_texture, input_sampler, in.uv + vec2<f32>(shift, 0.0)).r;
    
    // 暗角
    let center = vec2<f32>(0.5, 0.5);
    let vignette = 1.0 - length(in.uv - center) * 0.8;
    color.rgb *= vignette;
    
    return color;
}

// 热视觉效果
@fragment
fn fs_thermal_vision(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    let luminance = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    
    // 热感应颜色映射
    var thermal_color: vec3<f32>;
    
    if (luminance < 0.25) {
        thermal_color = mix(vec3<f32>(0.0, 0.0, 0.5), vec3<f32>(0.0, 0.0, 1.0), luminance * 4.0);
    } else if (luminance < 0.5) {
        thermal_color = mix(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 0.0), (luminance - 0.25) * 4.0);
    } else if (luminance < 0.75) {
        thermal_color = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), (luminance - 0.5) * 4.0);
    } else {
        thermal_color = mix(vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), (luminance - 0.75) * 4.0);
    }
    
    return vec4<f32>(thermal_color * uniforms.intensity, color.a);
}
