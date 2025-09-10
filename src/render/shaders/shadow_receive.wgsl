// 阴影接收着色器

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) light_space_position: vec4<f32>,
};

struct CameraUniforms {
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    camera_position: vec3<f32>,
    _padding: f32,
};

struct ModelUniforms {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

struct LightUniforms {
    position: vec3<f32>,
    _padding1: f32,
    direction: vec3<f32>,
    _padding2: f32,
    color: vec3<f32>,
    intensity: f32,
    light_type: u32, // 0=directional, 1=point, 2=spot
    range: f32,
    spot_angle: f32,
    _padding3: f32,
};

struct ShadowUniforms {
    light_space_matrix: mat4x4<f32>,
    light_position: vec4<f32>,
    shadow_bias: f32,
    normal_bias: f32,
    cascade_count: u32,
    _padding: u32,
    cascade_distances: array<f32, 4>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(1) @binding(0)
var<uniform> model: ModelUniforms;

@group(2) @binding(0)
var<uniform> light: LightUniforms;

@group(3) @binding(0)
var<uniform> shadow_uniforms: ShadowUniforms;

@group(3) @binding(1)
var shadow_map: texture_depth_2d;

@group(3) @binding(2)
var shadow_sampler: sampler_comparison;

@group(4) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(4) @binding(1)
var base_color_sampler: sampler;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_position = model.model_matrix * vec4<f32>(vertex.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.projection_matrix * camera.view_matrix * world_position;
    
    // 计算世界空间法线
    out.world_normal = normalize((model.normal_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    
    // 传递UV坐标
    out.uv = vertex.uv;
    
    // 计算光源空间位置
    out.light_space_position = shadow_uniforms.light_space_matrix * world_position;
    
    return out;
}

// PCF（百分比滤波）阴影计算
fn calculate_shadow_pcf(shadow_coord: vec3<f32>, bias: f32) -> f32 {
    let texel_size = 1.0 / 2048.0; // 假设2048x2048阴影贴图
    var shadow = 0.0;
    
    // 3x3 PCF采样
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_coord = shadow_coord.xy + offset;
            shadow += textureSampleCompare(shadow_map, shadow_sampler, sample_coord, shadow_coord.z - bias);
        }
    }
    
    return shadow / 9.0;
}

// 硬阴影计算
fn calculate_shadow_hard(shadow_coord: vec3<f32>, bias: f32) -> f32 {
    return textureSampleCompare(shadow_map, shadow_sampler, shadow_coord.xy, shadow_coord.z - bias);
}

// 泊松圆盘软阴影
fn calculate_shadow_poisson(shadow_coord: vec3<f32>, bias: f32) -> f32 {
    let poisson_disk = array<vec2<f32>, 16>(
        vec2<f32>(-0.94201624, -0.39906216),
        vec2<f32>(0.94558609, -0.76890725),
        vec2<f32>(-0.094184101, -0.92938870),
        vec2<f32>(0.34495938, 0.29387760),
        vec2<f32>(-0.91588581, 0.45771432),
        vec2<f32>(-0.81544232, -0.87912464),
        vec2<f32>(-0.38277543, 0.27676845),
        vec2<f32>(0.97484398, 0.75648379),
        vec2<f32>(0.44323325, -0.97511554),
        vec2<f32>(0.53742981, -0.47373420),
        vec2<f32>(-0.26496911, -0.41893023),
        vec2<f32>(0.79197514, 0.19090188),
        vec2<f32>(-0.24188840, 0.99706507),
        vec2<f32>(-0.81409955, 0.91437590),
        vec2<f32>(0.19984126, 0.78641367),
        vec2<f32>(0.14383161, -0.14100790)
    );
    
    let texel_size = 1.0 / 2048.0;
    let disk_radius = 2.0 * texel_size;
    
    var shadow = 0.0;
    for (var i = 0; i < 16; i++) {
        let offset = poisson_disk[i] * disk_radius;
        let sample_coord = shadow_coord.xy + offset;
        shadow += textureSampleCompare(shadow_map, shadow_sampler, sample_coord, shadow_coord.z - bias);
    }
    
    return shadow / 16.0;
}

// 计算阴影
fn calculate_shadow(light_space_pos: vec4<f32>, world_normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    // 透视除法
    var shadow_coord = light_space_pos.xyz / light_space_pos.w;
    
    // 转换到[0,1]范围
    shadow_coord = shadow_coord * 0.5 + 0.5;
    shadow_coord.y = 1.0 - shadow_coord.y; // 翻转Y轴（如果需要）
    
    // 超出阴影贴图范围的区域不受阴影影响
    if (shadow_coord.x < 0.0 || shadow_coord.x > 1.0 || 
        shadow_coord.y < 0.0 || shadow_coord.y > 1.0 ||
        shadow_coord.z > 1.0) {
        return 1.0;
    }
    
    // 计算偏移量以减少阴影粉刺
    let n_dot_l = dot(world_normal, light_dir);
    let bias = shadow_uniforms.shadow_bias + shadow_uniforms.normal_bias * sqrt(1.0 - n_dot_l * n_dot_l);
    
    // 使用PCF进行软阴影计算
    return calculate_shadow_pcf(shadow_coord, bias);
}

// 简单的Blinn-Phong光照模型
fn calculate_lighting(
    world_pos: vec3<f32>,
    world_normal: vec3<f32>,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
    light_color: vec3<f32>,
    material_color: vec3<f32>
) -> vec3<f32> {
    let ambient = 0.1 * material_color;
    
    // 漫反射
    let n_dot_l = max(dot(world_normal, light_dir), 0.0);
    let diffuse = n_dot_l * light_color * material_color;
    
    // 镜面反射
    let half_dir = normalize(light_dir + view_dir);
    let n_dot_h = max(dot(world_normal, half_dir), 0.0);
    let specular = pow(n_dot_h, 32.0) * light_color;
    
    return ambient + diffuse + specular;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 采样基础颜色纹理
    let base_color = textureSample(base_color_texture, base_color_sampler, in.uv);
    
    // 计算光照方向
    var light_dir: vec3<f32>;
    if (light.light_type == 0u) {
        // 方向光
        light_dir = normalize(-light.direction);
    } else {
        // 点光源或聚光灯
        light_dir = normalize(light.position - in.world_position);
    }
    
    // 计算视线方向
    let view_dir = normalize(camera.camera_position - in.world_position);
    
    // 计算基础光照
    let lighting = calculate_lighting(
        in.world_position,
        normalize(in.world_normal),
        view_dir,
        light_dir,
        light.color * light.intensity,
        base_color.rgb
    );
    
    // 计算阴影
    let shadow_factor = calculate_shadow(
        in.light_space_position,
        normalize(in.world_normal),
        light_dir
    );
    
    // 应用阴影（保留环境光）
    let ambient = 0.1 * base_color.rgb;
    let lit_color = ambient + (lighting - ambient) * shadow_factor;
    
    return vec4<f32>(lit_color, base_color.a);
}
