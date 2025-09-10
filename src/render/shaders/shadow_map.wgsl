// 阴影贴图着色器

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
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
var<uniform> shadow_uniforms: ShadowUniforms;

struct ModelUniforms {
    model_matrix: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> model: ModelUniforms;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_position = model.model_matrix * vec4<f32>(vertex.position, 1.0);
    out.clip_position = shadow_uniforms.light_space_matrix * world_position;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 阴影贴图只需要写入深度，不需要颜色输出
    // 但某些情况下可能需要写入深度值到颜色缓冲区
    return vec4<f32>(in.clip_position.z, 0.0, 0.0, 1.0);
}
