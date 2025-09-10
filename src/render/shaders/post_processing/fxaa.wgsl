// FXAA (Fast Approximate Anti-Aliasing) 着色器

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct FXAAUniforms {
    texel_size: vec2<f32>,
    quality_preset: u32, // 0=Low, 1=Medium, 2=High, 3=Ultra
    _padding: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: FXAAUniforms;

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

// RGB到亮度转换
fn rgb_to_luma(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

// FXAA质量设置
fn get_quality_settings(preset: u32) -> array<f32, 12> {
    switch preset {
        case 0u: { // Low - 10 samples
            return array<f32, 12>(
                1.5, 3.0, 12.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
            );
        }
        case 1u: { // Medium - 15 samples
            return array<f32, 12>(
                1.5, 3.0, 12.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
            );
        }
        case 2u: { // High - 29 samples
            return array<f32, 12>(
                1.5, 3.0, 12.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
            );
        }
        case 3u: { // Ultra - 39 samples
            return array<f32, 12>(
                1.5, 3.0, 12.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
            );
        }
        default: {
            return array<f32, 12>(
                1.5, 3.0, 12.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
            );
        }
    }
}

// FXAA核心算法
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = uniforms.texel_size;
    let uv = in.uv;
    
    // FXAA常量
    let FXAA_EDGE_THRESHOLD = 0.166;
    let FXAA_EDGE_THRESHOLD_MIN = 0.0833;
    let FXAA_SUBPIX_TRIM = 0.25;
    let FXAA_SUBPIX_TRIM_SCALE = 1.0 / (1.0 - FXAA_SUBPIX_TRIM);
    let FXAA_SUBPIX_CAP = 0.75;
    
    // 采样中心像素和周围像素
    let rgbM = textureSample(input_texture, input_sampler, uv).rgb;
    let rgbNW = textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, -texel_size.y)).rgb;
    let rgbNE = textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, -texel_size.y)).rgb;
    let rgbSW = textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, texel_size.y)).rgb;
    let rgbSE = textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, texel_size.y)).rgb;
    
    // 转换为亮度
    let lumaM = rgb_to_luma(rgbM);
    let lumaNW = rgb_to_luma(rgbNW);
    let lumaNE = rgb_to_luma(rgbNE);
    let lumaSW = rgb_to_luma(rgbSW);
    let lumaSE = rgb_to_luma(rgbSE);
    
    // 计算亮度范围
    let lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    let lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));
    let lumaRange = lumaMax - lumaMin;
    
    // 如果对比度太低，跳过抗锯齿
    if (lumaRange < max(FXAA_EDGE_THRESHOLD_MIN, lumaMax * FXAA_EDGE_THRESHOLD)) {
        return vec4<f32>(rgbM, 1.0);
    }
    
    // 采样更多邻居像素
    let rgbN = textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, -texel_size.y)).rgb;
    let rgbS = textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, texel_size.y)).rgb;
    let rgbW = textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, 0.0)).rgb;
    let rgbE = textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, 0.0)).rgb;
    
    let lumaN = rgb_to_luma(rgbN);
    let lumaS = rgb_to_luma(rgbS);
    let lumaW = rgb_to_luma(rgbW);
    let lumaE = rgb_to_luma(rgbE);
    
    // 计算子像素混合因子
    let rgbL = (rgbN + rgbS + rgbW + rgbE) * 0.25;
    let lumaL = rgb_to_luma(rgbL);
    let rangeL = abs(lumaL - lumaM);
    let blendL = max(0.0, (rangeL / lumaRange) - FXAA_SUBPIX_TRIM) * FXAA_SUBPIX_TRIM_SCALE;
    let blendL_capped = min(FXAA_SUBPIX_CAP, blendL);
    
    // 计算边缘方向
    let edgeVert = abs((lumaNW + lumaN + lumaNE) - (lumaSW + lumaS + lumaSE));
    let edgeHorz = abs((lumaNW + lumaW + lumaSW) - (lumaNE + lumaE + lumaSE));
    let horzSpan = edgeHorz >= edgeVert;
    
    // 选择步长方向
    var lengthSign: f32;
    var lumaN1: f32;
    var lumaN2: f32;
    
    if (horzSpan) {
        lengthSign = texel_size.y;
        lumaN1 = lumaN;
        lumaN2 = lumaS;
    } else {
        lengthSign = texel_size.x;
        lumaN1 = lumaW;
        lumaN2 = lumaE;
    }
    
    // 计算梯度
    let gradientN1 = abs(lumaN1 - lumaM);
    let gradientN2 = abs(lumaN2 - lumaM);
    let gradientScaled = max(gradientN1, gradientN2) * 0.25;
    
    if (gradientN1 >= gradientN2) {
        lengthSign = -lengthSign;
    }
    
    // 在边缘方向上搜索
    var uv1: vec2<f32>;
    var uv2: vec2<f32>;
    
    if (horzSpan) {
        uv1 = uv + vec2<f32>(-texel_size.x, lengthSign * 0.5);
        uv2 = uv + vec2<f32>(texel_size.x, lengthSign * 0.5);
    } else {
        uv1 = uv + vec2<f32>(lengthSign * 0.5, -texel_size.y);
        uv2 = uv + vec2<f32>(lengthSign * 0.5, texel_size.y);
    }
    
    // 采样边缘点
    let luma1 = rgb_to_luma(textureSample(input_texture, input_sampler, uv1).rgb);
    let luma2 = rgb_to_luma(textureSample(input_texture, input_sampler, uv2).rgb);
    
    // 计算边缘长度
    let reached1 = abs(luma1 - lumaM) >= gradientScaled;
    let reached2 = abs(luma2 - lumaM) >= gradientScaled;
    let reachedBoth = reached1 && reached2;
    
    var distance1 = 1.0;
    var distance2 = 1.0;
    
    if (!reached1) {
        distance1 = 2.0; // 简化的距离计算
    }
    if (!reached2) {
        distance2 = 2.0; // 简化的距离计算
    }
    
    // 计算最终的混合因子
    let distanceMin = min(distance1, distance2);
    let pixelOffset = 0.5 - distanceMin / (distance1 + distance2);
    let pixelOffsetGood = pixelOffset > 0.0;
    let pixelOffsetSubpix = max(pixelOffsetGood ? pixelOffset : 0.0, blendL_capped);
    
    // 计算最终UV坐标
    var finalUV = uv;
    if (horzSpan) {
        finalUV.y += pixelOffsetSubpix * lengthSign;
    } else {
        finalUV.x += pixelOffsetSubpix * lengthSign;
    }
    
    // 返回最终颜色
    let finalColor = textureSample(input_texture, input_sampler, finalUV).rgb;
    return vec4<f32>(finalColor, 1.0);
}

// 简化版FXAA（更高性能）
@fragment
fn fs_fxaa_simple(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = uniforms.texel_size;
    let uv = in.uv;
    
    // 采样5个点
    let rgbM = textureSample(input_texture, input_sampler, uv).rgb;
    let rgbN = textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, -texel_size.y)).rgb;
    let rgbS = textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, texel_size.y)).rgb;
    let rgbW = textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, 0.0)).rgb;
    let rgbE = textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, 0.0)).rgb;
    
    // 计算亮度
    let lumaM = rgb_to_luma(rgbM);
    let lumaN = rgb_to_luma(rgbN);
    let lumaS = rgb_to_luma(rgbS);
    let lumaW = rgb_to_luma(rgbW);
    let lumaE = rgb_to_luma(rgbE);
    
    // 检测边缘
    let lumaMin = min(lumaM, min(min(lumaN, lumaS), min(lumaW, lumaE)));
    let lumaMax = max(lumaM, max(max(lumaN, lumaS), max(lumaW, lumaE)));
    let lumaRange = lumaMax - lumaMin;
    
    // 如果对比度太低，返回原色
    if (lumaRange < 0.1) {
        return vec4<f32>(rgbM, 1.0);
    }
    
    // 简单的边缘方向检测
    let edgeH = abs(lumaN + lumaS - 2.0 * lumaM);
    let edgeV = abs(lumaW + lumaE - 2.0 * lumaM);
    
    var blend_factor = 0.5;
    var offset = vec2<f32>(0.0);
    
    if (edgeH > edgeV) {
        // 水平边缘，垂直模糊
        offset.y = texel_size.y * (lumaN > lumaS ? -1.0 : 1.0) * blend_factor;
    } else {
        // 垂直边缘，水平模糊
        offset.x = texel_size.x * (lumaW > lumaE ? -1.0 : 1.0) * blend_factor;
    }
    
    // 混合颜色
    let blended_color = textureSample(input_texture, input_sampler, uv + offset).rgb;
    let final_color = mix(rgbM, blended_color, 0.5);
    
    return vec4<f32>(final_color, 1.0);
}
