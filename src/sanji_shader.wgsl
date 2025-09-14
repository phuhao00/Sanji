// Sanji引擎 - 基础着色器
// 用于渲染彩色三角形

// 顶点着色器输出
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

// 顶点着色器
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // 根据顶点索引选择位置和颜色
    if (in_vertex_index == 0u) {
        out.clip_position = vec4<f32>(0.0, 0.5, 0.0, 1.0);   // 上顶点
        out.color = vec3<f32>(1.0, 0.0, 0.0); // 红色
    } else if (in_vertex_index == 1u) {
        out.clip_position = vec4<f32>(-0.5, -0.5, 0.0, 1.0); // 左下顶点
        out.color = vec3<f32>(0.0, 1.0, 0.0); // 绿色
    } else {
        out.clip_position = vec4<f32>(0.5, -0.5, 0.0, 1.0);  // 右下顶点
        out.color = vec3<f32>(0.0, 0.0, 1.0); // 蓝色
    }
    
    return out;
}

// 片元着色器
@fragment 
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
