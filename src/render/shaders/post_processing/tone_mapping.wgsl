// 色调映射着色器

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct ToneMappingUniforms {
    exposure: f32,
    white_point: f32,
    tone_mapper_type: u32, // 0=Reinhard, 1=ACES, 2=Filmic, 3=Uncharted2
    _padding: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: ToneMappingUniforms;

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

// 线性到sRGB转换
fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    return select(
        pow(linear, vec3<f32>(1.0 / 2.4)) * 1.055 - 0.055,
        linear * 12.92,
        linear <= vec3<f32>(0.0031308)
    );
}

// sRGB到线性转换
fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    return select(
        pow((srgb + 0.055) / 1.055, vec3<f32>(2.4)),
        srgb / 12.92,
        srgb <= vec3<f32>(0.04045)
    );
}

// Reinhard色调映射
fn tone_map_reinhard(hdr_color: vec3<f32>, white_point: f32) -> vec3<f32> {
    let numerator = hdr_color * (1.0 + hdr_color / (white_point * white_point));
    return numerator / (1.0 + hdr_color);
}

// ACES色调映射（近似）
fn tone_map_aces(hdr_color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    
    return clamp((hdr_color * (a * hdr_color + b)) / (hdr_color * (c * hdr_color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// Filmic色调映射（John Hable）
fn tone_map_filmic(hdr_color: vec3<f32>) -> vec3<f32> {
    let A = 0.15; // 肩部强度
    let B = 0.50; // 线性强度
    let C = 0.10; // 线性角度
    let D = 0.20; // 脚趾强度
    let E = 0.02; // 脚趾分子
    let F = 0.30; // 脚趾分母
    
    let hable = |x: vec3<f32>| -> vec3<f32> {
        ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F
    };
    
    let white_scale = 1.0 / hable(vec3<f32>(11.2)); // 白点
    return hable(hdr_color) * white_scale;
}

// Uncharted 2色调映射
fn tone_map_uncharted2(hdr_color: vec3<f32>) -> vec3<f32> {
    let A = 0.15;
    let B = 0.50;
    let C = 0.10;
    let D = 0.20;
    let E = 0.02;
    let F = 0.30;
    let W = 11.2;
    
    let uncharted2_tonemap = |x: vec3<f32>| -> vec3<f32> {
        ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F
    };
    
    let curr = uncharted2_tonemap(hdr_color * 2.0);
    let white_scale = 1.0 / uncharted2_tonemap(vec3<f32>(W));
    
    return curr * white_scale;
}

// Lottes色调映射
fn tone_map_lottes(hdr_color: vec3<f32>) -> vec3<f32> {
    let a = 1.6;
    let d = 0.977;
    let hdr_max = 8.0;
    let mid_in = 0.18;
    let mid_out = 0.267;
    
    let b = (-pow(mid_in, a) + pow(hdr_max, a) * mid_out) / ((pow(hdr_max, a * d) - pow(mid_in, a * d)) * mid_out);
    let c = (pow(hdr_max, a * d) * pow(mid_in, a) - pow(hdr_max, a) * pow(mid_in, a * d) * mid_out) / ((pow(hdr_max, a * d) - pow(mid_in, a * d)) * mid_out);
    
    return pow(hdr_color, vec3<f32>(a)) / (pow(hdr_color, vec3<f32>(a * d)) * b + c);
}

// AMD AGX色调映射
fn tone_map_agx(hdr_color: vec3<f32>) -> vec3<f32> {
    let min_ev = -12.47393;
    let max_ev = 4.026069;
    
    // AGX变换矩阵
    let agx_mat = mat3x3<f32>(
        vec3<f32>(0.842479062253094, 0.0423282422610123, 0.0423756549057051),
        vec3<f32>(0.0536185406129163, 0.95308259537157, 0.0536185406129163),
        vec3<f32>(0.0521945654529654, 0.0521945654529654, 0.948716846043165)
    );
    
    let agx_mat_inv = mat3x3<f32>(
        vec3<f32>(1.19687900512017, -0.0528968517574562, -0.0529716355144438),
        vec3<f32>(-0.0654737142199135, 1.05521685938145, -0.0654737142199135),
        vec3<f32>(-0.0589055854779635, -0.0589055854779635, 1.11778209352273)
    );
    
    // 对数编码
    let hdr_log = log2(hdr_color);
    let agx_log = (hdr_log - min_ev) / (max_ev - min_ev);
    
    // AGX变换
    let agx_color = agx_mat * clamp(agx_log, vec3<f32>(0.0), vec3<f32>(1.0));
    
    // AGX反变换
    return agx_mat_inv * agx_color;
}

// 自适应色调映射
fn tone_map_adaptive(hdr_color: vec3<f32>, luminance: f32) -> vec3<f32> {
    let adapted_lum = 0.5; // 简化的自适应亮度
    let scaled_lum = luminance / adapted_lum;
    
    return hdr_color / (1.0 + scaled_lum);
}

// 颜色分级
fn apply_color_grading(color: vec3<f32>) -> vec3<f32> {
    // 这里可以添加颜色分级参数
    // 对比度、饱和度、色调偏移等
    return color;
}

// 伽马校正
fn apply_gamma_correction(color: vec3<f32>, gamma: f32) -> vec3<f32> {
    return pow(color, vec3<f32>(1.0 / gamma));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 采样HDR颜色
    let hdr_color = textureSample(input_texture, input_sampler, in.uv);
    
    // 应用曝光
    var exposed_color = hdr_color.rgb * uniforms.exposure;
    
    // 应用色调映射
    var tone_mapped_color: vec3<f32>;
    
    switch uniforms.tone_mapper_type {
        case 0u: {
            tone_mapped_color = tone_map_reinhard(exposed_color, uniforms.white_point);
        }
        case 1u: {
            tone_mapped_color = tone_map_aces(exposed_color);
        }
        case 2u: {
            tone_mapped_color = tone_map_filmic(exposed_color);
        }
        case 3u: {
            tone_mapped_color = tone_map_uncharted2(exposed_color);
        }
        default: {
            tone_mapped_color = tone_map_aces(exposed_color);
        }
    }
    
    // 应用颜色分级
    tone_mapped_color = apply_color_grading(tone_mapped_color);
    
    // 转换到sRGB色彩空间
    let final_color = linear_to_srgb(tone_mapped_color);
    
    return vec4<f32>(final_color, hdr_color.a);
}

// 自动曝光着色器（用于计算场景平均亮度）
@fragment
fn fs_auto_exposure(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    
    // 计算亮度（使用对数空间避免过亮像素的过度影响）
    let luminance = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let log_luminance = log(max(luminance, 0.0001));
    
    return vec4<f32>(log_luminance, log_luminance, log_luminance, 1.0);
}

// 直方图生成着色器
@fragment
fn fs_histogram(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    let luminance = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    
    // 将亮度映射到直方图bin
    let bin_index = u32(clamp(luminance * 255.0, 0.0, 255.0));
    
    // 这里需要原子操作来更新直方图，
    // 在实际实现中需要使用计算着色器
    return vec4<f32>(f32(bin_index) / 255.0, 0.0, 0.0, 1.0);
}
