// 色彩分级着色器

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct ColorGradingUniforms {
    contrast: f32,
    brightness: f32,
    saturation: f32,
    hue_shift: f32,
    shadows: vec3<f32>,
    _padding1: f32,
    midtones: vec3<f32>,
    _padding2: f32,
    highlights: vec3<f32>,
    _padding3: f32,
    lift: vec3<f32>,
    _padding4: f32,
    gamma: vec3<f32>,
    _padding5: f32,
    gain: vec3<f32>,
    _padding6: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: ColorGradingUniforms;

@group(0) @binding(1)
var input_texture: texture_2d<f32>;

@group(0) @binding(2)
var input_sampler: sampler;

@group(0) @binding(3)
var lut_texture: texture_3d<f32>; // 3D LUT纹理

@group(0) @binding(4)
var lut_sampler: sampler;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    return out;
}

// RGB到HSV转换
fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let c_max = max(rgb.r, max(rgb.g, rgb.b));
    let c_min = min(rgb.r, min(rgb.g, rgb.b));
    let delta = c_max - c_min;
    
    var h: f32;
    let s = select(0.0, delta / c_max, c_max != 0.0);
    let v = c_max;
    
    if (delta == 0.0) {
        h = 0.0;
    } else if (c_max == rgb.r) {
        h = ((rgb.g - rgb.b) / delta) % 6.0;
    } else if (c_max == rgb.g) {
        h = (rgb.b - rgb.r) / delta + 2.0;
    } else {
        h = (rgb.r - rgb.g) / delta + 4.0;
    }
    
    h *= 60.0;
    if (h < 0.0) {
        h += 360.0;
    }
    
    return vec3<f32>(h, s, v);
}

// HSV到RGB转换
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.x / 60.0;
    let s = hsv.y;
    let v = hsv.z;
    
    let c = v * s;
    let x = c * (1.0 - abs((h % 2.0) - 1.0));
    let m = v - c;
    
    var rgb: vec3<f32>;
    
    if (h >= 0.0 && h < 1.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h >= 1.0 && h < 2.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h >= 2.0 && h < 3.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h >= 3.0 && h < 4.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h >= 4.0 && h < 5.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }
    
    return rgb + m;
}

// 应用对比度
fn apply_contrast(color: vec3<f32>, contrast: f32) -> vec3<f32> {
    return (color - 0.5) * contrast + 0.5;
}

// 应用亮度
fn apply_brightness(color: vec3<f32>, brightness: f32) -> vec3<f32> {
    return color + brightness;
}

// 应用饱和度
fn apply_saturation(color: vec3<f32>, saturation: f32) -> vec3<f32> {
    let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    return mix(vec3<f32>(luminance), color, saturation);
}

// 应用色调偏移
fn apply_hue_shift(color: vec3<f32>, hue_shift: f32) -> vec3<f32> {
    let hsv = rgb_to_hsv(color);
    let new_hsv = vec3<f32>(hsv.x + hue_shift, hsv.y, hsv.z);
    return hsv_to_rgb(new_hsv);
}

// 计算亮度权重
fn get_luminance_weight(color: vec3<f32>) -> vec3<f32> {
    let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    
    // 阴影、中间调、高光权重计算
    let shadow_weight = 1.0 - smoothstep(0.0, 0.3, luminance);
    let highlight_weight = smoothstep(0.7, 1.0, luminance);
    let midtone_weight = 1.0 - shadow_weight - highlight_weight;
    
    return vec3<f32>(shadow_weight, midtone_weight, highlight_weight);
}

// 应用阴影、中间调、高光调整
fn apply_tone_adjustments(color: vec3<f32>) -> vec3<f32> {
    let weights = get_luminance_weight(color);
    
    let shadow_adjustment = uniforms.shadows * weights.x;
    let midtone_adjustment = uniforms.midtones * weights.y;
    let highlight_adjustment = uniforms.highlights * weights.z;
    
    return color * (shadow_adjustment + midtone_adjustment + highlight_adjustment);
}

// 应用Lift/Gamma/Gain调整
fn apply_lift_gamma_gain(color: vec3<f32>) -> vec3<f32> {
    // Lift（提升）- 影响阴影
    var adjusted_color = color + uniforms.lift;
    
    // Gamma（伽马）- 影响中间调
    adjusted_color = sign(adjusted_color) * pow(abs(adjusted_color), 1.0 / uniforms.gamma);
    
    // Gain（增益）- 影响高光
    adjusted_color = adjusted_color * uniforms.gain;
    
    return adjusted_color;
}

// 3D LUT查找
fn sample_3d_lut(color: vec3<f32>, lut_size: f32) -> vec3<f32> {
    let scaled = color * (lut_size - 1.0);
    let base_coord = floor(scaled) / (lut_size - 1.0);
    let next_coord = ceil(scaled) / (lut_size - 1.0);
    let blend_factor = fract(scaled);
    
    // 8个角点采样
    let c000 = textureSample(lut_texture, lut_sampler, vec3<f32>(base_coord.x, base_coord.y, base_coord.z)).rgb;
    let c001 = textureSample(lut_texture, lut_sampler, vec3<f32>(base_coord.x, base_coord.y, next_coord.z)).rgb;
    let c010 = textureSample(lut_texture, lut_sampler, vec3<f32>(base_coord.x, next_coord.y, base_coord.z)).rgb;
    let c011 = textureSample(lut_texture, lut_sampler, vec3<f32>(base_coord.x, next_coord.y, next_coord.z)).rgb;
    let c100 = textureSample(lut_texture, lut_sampler, vec3<f32>(next_coord.x, base_coord.y, base_coord.z)).rgb;
    let c101 = textureSample(lut_texture, lut_sampler, vec3<f32>(next_coord.x, base_coord.y, next_coord.z)).rgb;
    let c110 = textureSample(lut_texture, lut_sampler, vec3<f32>(next_coord.x, next_coord.y, base_coord.z)).rgb;
    let c111 = textureSample(lut_texture, lut_sampler, vec3<f32>(next_coord.x, next_coord.y, next_coord.z)).rgb;
    
    // 三线性插值
    let c00 = mix(c000, c001, blend_factor.z);
    let c01 = mix(c010, c011, blend_factor.z);
    let c10 = mix(c100, c101, blend_factor.z);
    let c11 = mix(c110, c111, blend_factor.z);
    
    let c0 = mix(c00, c01, blend_factor.y);
    let c1 = mix(c10, c11, blend_factor.y);
    
    return mix(c0, c1, blend_factor.x);
}

// 温度和色调调整
fn apply_temperature_tint(color: vec3<f32>, temperature: f32, tint: f32) -> vec3<f32> {
    // 色温调整矩阵（简化版）
    let temp_matrix = mat3x3<f32>(
        vec3<f32>(1.0 + temperature * 0.1, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0 - temperature * 0.1)
    );
    
    // 色调调整
    let tint_matrix = mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0 + tint * 0.1, 0.0),
        vec3<f32>(0.0, 0.0, 1.0 - tint * 0.1)
    );
    
    return tint_matrix * temp_matrix * color;
}

// 颜色平衡调整
fn apply_color_balance(color: vec3<f32>, shadows: vec3<f32>, midtones: vec3<f32>, highlights: vec3<f32>) -> vec3<f32> {
    let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
    
    // 计算权重
    let shadow_weight = 1.0 - smoothstep(0.0, 0.3, luminance);
    let highlight_weight = smoothstep(0.7, 1.0, luminance);
    let midtone_weight = 1.0 - shadow_weight - highlight_weight;
    
    // 应用颜色平衡
    let balance = shadows * shadow_weight + midtones * midtone_weight + highlights * highlight_weight;
    
    return color + balance;
}

// Vignette效果
fn apply_vignette(color: vec3<f32>, uv: vec2<f32>, intensity: f32, smoothness: f32) -> vec3<f32> {
    let center = vec2<f32>(0.5, 0.5);
    let distance = length(uv - center);
    let vignette = smoothstep(0.5, 0.5 - smoothness, distance * intensity);
    return color * vignette;
}

// 胶片颗粒效果
fn apply_film_grain(color: vec3<f32>, uv: vec2<f32>, strength: f32, time: f32) -> vec3<f32> {
    // 简单的噪声函数
    let noise = fract(sin(dot(uv + time, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let grain = (noise - 0.5) * strength;
    
    return color + grain;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 采样输入颜色
    var color = textureSample(input_texture, input_sampler, in.uv).rgb;
    
    // 应用基础调整
    color = apply_brightness(color, uniforms.brightness);
    color = apply_contrast(color, uniforms.contrast);
    color = apply_saturation(color, uniforms.saturation);
    color = apply_hue_shift(color, uniforms.hue_shift);
    
    // 应用色调调整
    color = apply_tone_adjustments(color);
    
    // 应用Lift/Gamma/Gain
    color = apply_lift_gamma_gain(color);
    
    // 应用3D LUT（如果有）
    // color = sample_3d_lut(color, 32.0);
    
    // 确保颜色在有效范围内
    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
    
    return vec4<f32>(color, 1.0);
}

// 实时颜色分级（使用预设）
@fragment
fn fs_color_grade_preset(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv).rgb;
    
    // 电影风格预设
    var graded_color = color;
    
    // 增加对比度
    graded_color = apply_contrast(graded_color, 1.2);
    
    // 轻微降低饱和度
    graded_color = apply_saturation(graded_color, 0.9);
    
    // 温暖色调
    graded_color = apply_temperature_tint(graded_color, 0.1, -0.05);
    
    // 提升阴影，压低高光
    let weights = get_luminance_weight(graded_color);
    let shadow_lift = vec3<f32>(0.05, 0.03, 0.01);
    let highlight_gain = vec3<f32>(0.95, 0.97, 0.99);
    
    graded_color = graded_color + shadow_lift * weights.x;
    graded_color = graded_color * (highlight_gain * weights.z + vec3<f32>(1.0) * (1.0 - weights.z));
    
    // 轻微暗角效果
    graded_color = apply_vignette(graded_color, in.uv, 0.8, 0.5);
    
    return vec4<f32>(clamp(graded_color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
