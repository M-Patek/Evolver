use rug::{Integer, Float};
use rug::float::Constant;
use crate::soul::algebra::ClassGroupElement; // 假设这是代数结构的定义位置

/// 投影器配置
pub struct ProjectionConfig {
    pub precision: u32,
}

/// 拓扑投影器 (Topological Projector)
/// 
/// 负责将离散的代数状态 (Algebraic State) 投影到连续的几何流形 (Manifold)。
/// VAPO 依赖这个投影产生的“地形”来寻找方向。
pub struct TopologicalProjector {
    precision: u32,
}

impl TopologicalProjector {
    pub fn new(config: ProjectionConfig) -> Self {
        Self {
            precision: config.precision,
        }
    }

    /// 周期性特征投影 (Periodic Feature Projection)
    /// 
    /// [AUDIT FIX]: 修复了大数求三角函数的随机噪声问题。
    /// 
    /// 原理：
    /// 如果直接计算 sin(Norm)，当 Norm ~ 10^180 时，结果完全取决于浮点数
    /// 末尾的垃圾位，等同于随机数。
    /// 
    /// 修正逻辑：
    /// 1. 在代数域 (Integer Domain) 进行模归约： reduced = norm % modulus
    /// 2. 在几何域 (Real Domain) 进行映射： phase = reduced / modulus * 2pi
    /// 3. 计算 sin(phase)
    /// 
    /// 这样保证了 Lipschiz 连续性，让 VAPO 即使在深空也能感知到平滑的坡度。
    pub fn project_periodic(&self, norm: &Integer, modulus: &Integer) -> Float {
        // Step 1: 代数域归约 (Exact Reduction)
        // 这是在纯整数算术中完成的，没有任何精度损失。
        let reduced = norm.clone() % modulus;
        
        // Step 2: 转换到几何域 (Safe Float Conversion)
        // 此时 reduced < modulus，通常 modulus 是预设的常数或相对较小的数，
        // 转换为 Float 是安全的。
        let f_reduced = Float::with_val(self.precision, reduced);
        let f_modulus = Float::with_val(self.precision, modulus);
        let pi = Float::with_val(self.precision, Constant::Pi);
        
        // Step 3: 计算相位 (Phase Calculation)
        // phase = (reduced / modulus) * 2 * pi
        let phase = (f_reduced / f_modulus) * 2.0 * pi;
        
        // Step 4: 计算正弦值
        phase.sin()
    }

    /// 对数特征投影 (Logarithmic Feature Projection)
    /// 
    /// 用于感知数量级的变化。
    /// 同样需要高精度，以捕捉 log(10^180 + delta) - log(10^180) 的微小差异。
    pub fn project_log_norm(&self, norm: &Integer) -> Float {
        // 将巨大的 Integer 转换为高精度 Float
        let f_norm = Float::with_val(self.precision, norm);
        
        // 计算自然对数
        f_norm.ln()
    }
    
    /// 综合投影 (Composite Projection)
    /// 
    /// 将多个特征组合成一个高维特征向量。
    pub fn project(&self, element: &ClassGroupElement, modulus_pool: &[Integer]) -> Vec<Float> {
        let norm = &element.norm; // 假设 element 包含一个巨大的理想范数
        let mut features = Vec::new();

        // 1. 添加对数特征 (感知距离)
        features.push(self.project_log_norm(norm));

        // 2. 添加周期性特征 (感知方向/角度)
        for modulus in modulus_pool {
            features.push(self.project_periodic(norm, modulus));
        }

        features
    }
}
