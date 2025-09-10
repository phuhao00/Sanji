// 级联阴影贴图(CSM)着色器

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
    @location(3) view_position: vec3<f32>,
};

struct CameraUniforms {
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    camera_position: vec3<f32>,
    near_plane: f32,
    far_plane: f32,
    _padding: array<f32, 3>,
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
    light_type: u32,
    range: f32,
    spot_angle: f32,
    _padding3: f32,
};

struct CSMUniforms {
    light_space_matrices: array<mat4x4<f32>, 4>,
    cascade_distances: array<f32, 4>,
    cascade_count: u32,
    shadow_bias: f32,
    normal_bias: f32,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(1) @binding(0)
var<uniform> model: ModelUniforms;

@group(2) @binding(0)
var<uniform> light: LightUniforms;

@group(3) @binding(0)
var<uniform> csm: CSMUniforms;

@group(3) @binding(1)
var cascade_0: texture_depth_2d;

@group(3) @binding(2)
var cascade_1: texture_depth_2d;

@group(3) @binding(3)
var cascade_2: texture_depth_2d;

@group(3) @binding(4)
var cascade_3: texture_depth_2d;

@group(3) @binding(5)
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
    
    // 计算视图空间位置（用于级联选择）
    out.view_position = (camera.view_matrix * world_position).xyz;
    
    // 计算世界空间法线
    out.world_normal = normalize((model.normal_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);
    
    // 传递UV坐标
    out.uv = vertex.uv;
    
    return out;
}

// 选择合适的级联
fn select_cascade(view_z: f32) -> u32 {
    let depth = -view_z; // 视图空间Z是负数
    
    for (var i: u32 = 0u; i < csm.cascade_count; i++) {
        if (depth <= csm.cascade_distances[i]) {
            return i;
        }
    }
    
    return csm.cascade_count - 1u;
}

// 计算指定级联的阴影
fn calculate_cascade_shadow(
    cascade_index: u32,
    world_pos: vec3<f32>,
    world_normal: vec3<f32>,
    light_dir: vec3<f32>
) -> f32 {
    // 计算光源空间位置
    let light_space_pos = csm.light_space_matrices[cascade_index] * vec4<f32>(world_pos, 1.0);
    
    // 透视除法
    var shadow_coord = light_space_pos.xyz / light_space_pos.w;
    
    // 转换到[0,1]范围
    shadow_coord = shadow_coord * 0.5 + 0.5;
    shadow_coord.y = 1.0 - shadow_coord.y;
    
    // 检查是否在阴影贴图范围内
    if (shadow_coord.x < 0.0 || shadow_coord.x > 1.0 || 
        shadow_coord.y < 0.0 || shadow_coord.y > 1.0 ||
        shadow_coord.z > 1.0) {
        return 1.0;
    }
    
    // 计算偏移量
    let n_dot_l = dot(world_normal, light_dir);
    let bias = csm.shadow_bias + csm.normal_bias * sqrt(1.0 - n_dot_l * n_dot_l);
    
    // 根据级联索引选择对应的阴影贴图
    var shadow_value: f32;
    
    switch cascade_index {
        case 0u: {
            shadow_value = sample_shadow_pcf(cascade_0, shadow_coord, bias);
        }
        case 1u: {
            shadow_value = sample_shadow_pcf(cascade_1, shadow_coord, bias);
        }
        case 2u: {
            shadow_value = sample_shadow_pcf(cascade_2, shadow_coord, bias);
        }
        case 3u: {
            shadow_value = sample_shadow_pcf(cascade_3, shadow_coord, bias);
        }
        default: {
            shadow_value = 1.0;
        }
    }
    
    return shadow_value;
}

// PCF阴影采样
fn sample_shadow_pcf(shadow_map: texture_depth_2d, shadow_coord: vec3<f32>, bias: f32) -> f32 {
    let texel_size = 1.0 / 2048.0;
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

// 级联边界混合
fn blend_cascade_shadows(
    cascade_index: u32,
    world_pos: vec3<f32>,
    world_normal: vec3<f32>,
    light_dir: vec3<f32>,
    view_z: f32
) -> f32 {
    let current_shadow = calculate_cascade_shadow(cascade_index, world_pos, world_normal, light_dir);
    
    // 如果不是最后一个级联，考虑与下一个级联的混合
    if (cascade_index < csm.cascade_count - 1u) {
        let depth = -view_z;
        let current_distance = csm.cascade_distances[cascade_index];
        let blend_distance = current_distance * 0.9; // 在90%距离处开始混合
        
        if (depth > blend_distance) {
            let next_shadow = calculate_cascade_shadow(cascade_index + 1u, world_pos, world_normal, light_dir);
            let blend_factor = (depth - blend_distance) / (current_distance - blend_distance);
            return mix(current_shadow, next_shadow, clamp(blend_factor, 0.0, 1.0));
        }
    }
    
    return current_shadow;
}

// 可视化级联颜色（调试用）
fn get_cascade_debug_color(cascade_index: u32) -> vec3<f32> {
    switch cascade_index {
        case 0u: { return vec3<f32>(1.0, 0.0, 0.0); } // 红色
        case 1u: { return vec3<f32>(0.0, 1.0, 0.0); } // 绿色
        case 2u: { return vec3<f32>(0.0, 0.0, 1.0); } // 蓝色
        case 3u: { return vec3<f32>(1.0, 1.0, 0.0); } // 黄色
        default: { return vec3<f32>(1.0, 1.0, 1.0); } // 白色
    }
}

// 简单的光照计算
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
    
    // 选择合适的级联
    let cascade_index = select_cascade(in.view_position.z);
    
    // 计算阴影（带级联混合）
    let shadow_factor = blend_cascade_shadows(
        cascade_index,
        in.world_position,
        normalize(in.world_normal),
        light_dir,
        in.view_position.z
    );
    
    // 应用阴影
    let ambient = 0.1 * base_color.rgb;
    let lit_color = ambient + (lighting - ambient) * shadow_factor;
    
    // 可选：显示级联调试颜色
    // let debug_color = get_cascade_debug_color(cascade_index);
    // let final_color = mix(lit_color, debug_color * 0.3, 0.3);
    
    return vec4<f32>(lit_color, base_color.a);
}
