//! 数学工具库模块

pub mod bounds;
pub mod ray;
pub mod frustum;
pub mod intersect;
pub mod noise;
pub mod easing;

pub use bounds::*;
pub use ray::*;
pub use frustum::*;
pub use intersect::*;
pub use noise::*;
pub use easing::*;

// 重新导出glam的常用类型
pub use glam::{
    Vec2, Vec3, Vec4, 
    IVec2, IVec3, IVec4,
    UVec2, UVec3, UVec4,
    Mat2, Mat3, Mat4,
    Quat,
};

/// 数学常量
pub mod consts {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const E: f32 = std::f32::consts::E;
    pub const SQRT_2: f32 = std::f32::consts::SQRT_2;
    pub const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
    pub const FRAC_PI_3: f32 = std::f32::consts::FRAC_PI_3;
    pub const FRAC_PI_4: f32 = std::f32::consts::FRAC_PI_4;
    pub const FRAC_PI_6: f32 = std::f32::consts::FRAC_PI_6;
    pub const FRAC_PI_8: f32 = std::f32::consts::FRAC_PI_8;
}

/// 角度转弧度
pub fn deg_to_rad(degrees: f32) -> f32 {
    degrees * consts::PI / 180.0
}

/// 弧度转角度
pub fn rad_to_deg(radians: f32) -> f32 {
    radians * 180.0 / consts::PI
}

/// 钳制值到指定范围
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// 线性插值
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// 反向线性插值
pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if (b - a).abs() < f32::EPSILON {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

/// 重映射值从一个范围到另一个范围
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = inverse_lerp(from_min, from_max, value);
    lerp(to_min, to_max, t)
}

/// 平滑阶跃函数
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 更平滑的阶跃函数
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Vec3扩展trait
pub trait Vec3Ext {
    /// 创建随机单位向量
    fn random_unit() -> Vec3;
    
    /// 创建随机方向向量(在球面上)
    fn random_on_sphere() -> Vec3;
    
    /// 创建随机方向向量(在半球上)
    fn random_on_hemisphere(normal: Vec3) -> Vec3;
    
    /// 沿着法线反射
    fn reflect(&self, normal: Vec3) -> Vec3;
    
    /// 沿着法线折射
    fn refract(&self, normal: Vec3, eta: f32) -> Vec3;
}

impl Vec3Ext for Vec3 {
    fn random_unit() -> Vec3 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        loop {
            let x = rng.gen_range(-1.0..1.0);
            let y = rng.gen_range(-1.0..1.0);
            let z = rng.gen_range(-1.0..1.0);
            let v = Vec3::new(x, y, z);
            
            if v.length_squared() <= 1.0 {
                return v.normalize_or_zero();
            }
        }
    }
    
    fn random_on_sphere() -> Vec3 {
        Self::random_unit()
    }
    
    fn random_on_hemisphere(normal: Vec3) -> Vec3 {
        let on_sphere = Self::random_on_sphere();
        if on_sphere.dot(normal) > 0.0 {
            on_sphere
        } else {
            -on_sphere
        }
    }
    
    fn reflect(&self, normal: Vec3) -> Vec3 {
        *self - 2.0 * self.dot(normal) * normal
    }
    
    fn refract(&self, normal: Vec3, eta: f32) -> Vec3 {
        let cos_theta = (-*self).dot(normal).min(1.0);
        let r_out_perp = eta * (*self + cos_theta * normal);
        let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * normal;
        r_out_perp + r_out_parallel
    }
}
