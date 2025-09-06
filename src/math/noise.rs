//! 噪声生成工具

use glam::{Vec2, Vec3};

/// 简单的伪随机数生成器
pub struct SimpleRng {
    seed: u32,
}

impl SimpleRng {
    /// 创建新的随机数生成器
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// 生成下一个随机数
    pub fn next(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        self.seed
    }

    /// 生成0-1之间的浮点数
    pub fn next_f32(&mut self) -> f32 {
        (self.next() as f32) / (u32::MAX as f32)
    }

    /// 生成指定范围内的浮点数
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }
}

/// Perlin噪声生成器
pub struct PerlinNoise {
    permutation: [u8; 512],
}

impl PerlinNoise {
    /// 创建新的Perlin噪声生成器
    pub fn new(seed: u32) -> Self {
        let mut rng = SimpleRng::new(seed);
        let mut p = [0u8; 256];
        
        // 初始化置换表
        for i in 0..256 {
            p[i] = i as u8;
        }
        
        // 随机打乱
        for i in (1..256).rev() {
            let j = (rng.next() as usize) % (i + 1);
            p.swap(i, j);
        }
        
        // 复制到512长度数组
        let mut permutation = [0u8; 512];
        for i in 0..512 {
            permutation[i] = p[i % 256];
        }
        
        Self { permutation }
    }

    /// 淡化函数
    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    /// 线性插值
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    /// 梯度函数
    fn grad(hash: u8, x: f32, y: f32, z: f32) -> f32 {
        let h = hash & 15;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
        
        (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
    }

    /// 生成1D Perlin噪声
    pub fn noise_1d(&self, x: f32) -> f32 {
        self.noise_3d(x, 0.0, 0.0)
    }

    /// 生成2D Perlin噪声
    pub fn noise_2d(&self, x: f32, y: f32) -> f32 {
        self.noise_3d(x, y, 0.0)
    }

    /// 生成3D Perlin噪声
    pub fn noise_3d(&self, x: f32, y: f32, z: f32) -> f32 {
        // 找到单位立方体中的坐标
        let xi = (x.floor() as i32) & 255;
        let yi = (y.floor() as i32) & 255;
        let zi = (z.floor() as i32) & 255;

        // 在立方体内的相对坐标
        let x = x - x.floor();
        let y = y - y.floor();
        let z = z - z.floor();

        // 淡化曲线
        let u = Self::fade(x);
        let v = Self::fade(y);
        let w = Self::fade(z);

        // 计算立方体8个角的哈希坐标
        let a = self.permutation[(xi as usize)] as usize + yi as usize;
        let aa = self.permutation[a] as usize + zi as usize;
        let ab = self.permutation[a + 1] as usize + zi as usize;
        let b = self.permutation[(xi as usize) + 1] as usize + yi as usize;
        let ba = self.permutation[b] as usize + zi as usize;
        let bb = self.permutation[b + 1] as usize + zi as usize;

        // 插值计算
        Self::lerp(
            Self::lerp(
                Self::lerp(
                    Self::grad(self.permutation[aa], x, y, z),
                    Self::grad(self.permutation[ba], x - 1.0, y, z),
                    u,
                ),
                Self::lerp(
                    Self::grad(self.permutation[ab], x, y - 1.0, z),
                    Self::grad(self.permutation[bb], x - 1.0, y - 1.0, z),
                    u,
                ),
                v,
            ),
            Self::lerp(
                Self::lerp(
                    Self::grad(self.permutation[aa + 1], x, y, z - 1.0),
                    Self::grad(self.permutation[ba + 1], x - 1.0, y, z - 1.0),
                    u,
                ),
                Self::lerp(
                    Self::grad(self.permutation[ab + 1], x, y - 1.0, z - 1.0),
                    Self::grad(self.permutation[bb + 1], x - 1.0, y - 1.0, z - 1.0),
                    u,
                ),
                v,
            ),
            w,
        )
    }

    /// 分形布朗运动(fBM)
    pub fn fbm_2d(&self, mut x: f32, mut y: f32, octaves: i32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let frequency = 1.0;
        let lacunarity = 2.0;
        let persistence = 0.5;

        for _ in 0..octaves {
            value += amplitude * self.noise_2d(x * frequency, y * frequency);
            x *= lacunarity;
            y *= lacunarity;
            amplitude *= persistence;
        }

        value
    }

    /// 脊形噪声
    pub fn ridge_noise_2d(&self, x: f32, y: f32, octaves: i32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let lacunarity = 2.0;
        let persistence = 0.5;

        for _ in 0..octaves {
            let n = self.noise_2d(x * frequency, y * frequency).abs();
            value += amplitude * (1.0 - n);
            frequency *= lacunarity;
            amplitude *= persistence;
        }

        value
    }
}

/// 简单噪声函数
pub struct SimpleNoise;

impl SimpleNoise {
    /// 白噪声
    pub fn white_noise(seed: u32) -> f32 {
        let mut rng = SimpleRng::new(seed);
        rng.next_f32() * 2.0 - 1.0
    }

    /// 值噪声 (简化版)
    pub fn value_noise_2d(x: f32, y: f32) -> f32 {
        let i = x.floor() as i32;
        let j = y.floor() as i32;
        
        let fx = x - x.floor();
        let fy = y - y.floor();
        
        // 简单哈希函数
        let hash = |x: i32, y: i32| -> f32 {
            let mut n = x.wrapping_mul(3) ^ y.wrapping_mul(113);
            n = (n << 13) ^ n;
            ((n.wrapping_mul(n.wrapping_mul(15731) + 789221) + 1376312589) & 0x7fffffff) as f32 / 1073741824.0 - 1.0
        };
        
        let a = hash(i, j);
        let b = hash(i + 1, j);
        let c = hash(i, j + 1);
        let d = hash(i + 1, j + 1);
        
        let i1 = crate::math::lerp(a, b, fx);
        let i2 = crate::math::lerp(c, d, fx);
        
        crate::math::lerp(i1, i2, fy)
    }

    /// Worley/Voronoi 噪声 (简化版)
    pub fn worley_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
        let cell_x = x.floor() as i32;
        let cell_y = y.floor() as i32;
        
        let mut min_distance = f32::INFINITY;
        
        for i in -1..=1 {
            for j in -1..=1 {
                let neighbor_x = cell_x + i;
                let neighbor_y = cell_y + j;
                
                // 生成邻居单元格中的随机点
                let mut rng = SimpleRng::new(
                    (neighbor_x as u32).wrapping_mul(374761393)
                    .wrapping_add((neighbor_y as u32).wrapping_mul(668265263))
                    .wrapping_add(seed)
                );
                
                let point_x = neighbor_x as f32 + rng.next_f32();
                let point_y = neighbor_y as f32 + rng.next_f32();
                
                let dx = x - point_x;
                let dy = y - point_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                min_distance = min_distance.min(distance);
            }
        }
        
        min_distance
    }
}

/// 噪声工具函数
pub struct NoiseUtils;

impl NoiseUtils {
    /// 重新映射噪声值到指定范围
    pub fn remap(value: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
        new_min + (new_max - new_min) * ((value - old_min) / (old_max - old_min))
    }

    /// 将噪声值钳制到0-1范围
    pub fn normalize(value: f32) -> f32 {
        (value + 1.0) * 0.5
    }

    /// 应用幂函数调整噪声对比度
    pub fn power(value: f32, exponent: f32) -> f32 {
        if value >= 0.0 {
            value.powf(exponent)
        } else {
            -(-value).powf(exponent)
        }
    }

    /// 应用阶跃函数
    pub fn step(value: f32, threshold: f32) -> f32 {
        if value >= threshold { 1.0 } else { 0.0 }
    }

    /// 平滑阶跃函数
    pub fn smoothstep(value: f32, edge0: f32, edge1: f32) -> f32 {
        let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}
