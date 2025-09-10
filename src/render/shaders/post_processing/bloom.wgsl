// Bloom后处理着色器

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BloomUniforms {
    threshold: f32,
    intensity: f32,
    radius: f32,
    _padding: f32,
    texel_size: vec2<f32>,
    _padding2: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: BloomUniforms;

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

// 亮度提取着色器
@fragment
fn fs_threshold(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.uv);
    
    // 计算亮度
    let luminance = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    
    // 应用阈值
    let brightness = max(luminance - uniforms.threshold, 0.0);
    let contribution = brightness / (brightness + 1.0);
    
    return vec4<f32>(color.rgb * contribution, color.a);
}

// 下采样着色器（13-tap降采样）
@fragment
fn fs_downsample(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = uniforms.texel_size;
    let uv = in.uv;
    
    // 13-tap降采样模式
    var color = textureSample(input_texture, input_sampler, uv) * 0.125;
    
    // 4个角点采样
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, -texel_size.y)) * 0.0625;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, -texel_size.y)) * 0.0625;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, texel_size.y)) * 0.0625;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, texel_size.y)) * 0.0625;
    
    // 4个边缘中点采样
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, 0.0)) * 0.125;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, 0.0)) * 0.125;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, -texel_size.y)) * 0.125;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, texel_size.y)) * 0.125;
    
    // 4个对角中点采样
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x * 0.5, -texel_size.y * 0.5)) * 0.0625;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x * 0.5, -texel_size.y * 0.5)) * 0.0625;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x * 0.5, texel_size.y * 0.5)) * 0.0625;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x * 0.5, texel_size.y * 0.5)) * 0.0625;
    
    return color;
}

// 上采样着色器（tent滤波）
@fragment
fn fs_upsample(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = uniforms.texel_size * uniforms.radius;
    let uv = in.uv;
    
    // 9-tap tent滤波
    var color = textureSample(input_texture, input_sampler, uv) * 0.25;
    
    // 4个直接邻居
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, 0.0)) * 0.125;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, 0.0)) * 0.125;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, -texel_size.y)) * 0.125;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, texel_size.y)) * 0.125;
    
    // 4个对角邻居
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, -texel_size.y)) * 0.0625;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, -texel_size.y)) * 0.0625;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, texel_size.y)) * 0.0625;
    color += textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, texel_size.y)) * 0.0625;
    
    return color;
}

// 高斯模糊（水平）
@fragment
fn fs_blur_horizontal(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = uniforms.texel_size;
    let uv = in.uv;
    
    // 9-tap高斯模糊
    let weights = array<f32, 9>(
        0.013519569015984728,
        0.047662179108871855,
        0.11723004402070096,
        0.20116755999375591,
        0.24197072451914337,
        0.20116755999375591,
        0.11723004402070096,
        0.047662179108871855,
        0.013519569015984728
    );
    
    var color = vec4<f32>(0.0);
    
    for (var i = -4; i <= 4; i++) {
        let offset = vec2<f32>(f32(i) * texel_size.x, 0.0);
        color += textureSample(input_texture, input_sampler, uv + offset) * weights[i + 4];
    }
    
    return color;
}

// 高斯模糊（垂直）
@fragment
fn fs_blur_vertical(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = uniforms.texel_size;
    let uv = in.uv;
    
    // 9-tap高斯模糊
    let weights = array<f32, 9>(
        0.013519569015984728,
        0.047662179108871855,
        0.11723004402070096,
        0.20116755999375591,
        0.24197072451914337,
        0.20116755999375591,
        0.11723004402070096,
        0.047662179108871855,
        0.013519569015984728
    );
    
    var color = vec4<f32>(0.0);
    
    for (var i = -4; i <= 4; i++) {
        let offset = vec2<f32>(0.0, f32(i) * texel_size.y);
        color += textureSample(input_texture, input_sampler, uv + offset) * weights[i + 4];
    }
    
    return color;
}

// Bloom合成着色器
@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(input_texture, input_sampler, in.uv);
    
    // 这里假设有第二个纹理绑定来获取bloom结果
    // 在实际实现中需要额外的绑定组
    // let bloom_color = textureSample(bloom_texture, input_sampler, in.uv);
    
    // 简化版本：直接返回处理过的颜色
    return vec4<f32>(base_color.rgb * uniforms.intensity, base_color.a);
}
