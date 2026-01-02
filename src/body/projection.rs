use crate::soul::algebra::IdealClass;
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use num_traits::ToPrimitive;
use num_bigint::BigInt;

/// NavigationFeatures (导航特征)
/// 用于启发式搜索的平滑几何特征。
#[derive(Debug, Clone)]
struct NavigationFeatures {
    pub cos_x: f64,
    pub sin_x: f64,
    pub log_y: f64,
}

impl NavigationFeatures {
    fn extract(state: &IdealClass) -> Self {
        let a_f64 = state.a.to_f64().unwrap_or(f64::MAX); 
        let b_f64 = state.b.to_f64().unwrap_or(f64::MAX);
        
        // Calculate Discriminant on the fly
        let four = BigInt::from(4);
        let b_sq = &state.b * &state.b;
        let ac4 = &four * &state.a * &state.c;
        let delta = b_sq - ac4;
        let discrim_f64 = delta.to_f64().unwrap_or(f64::MIN); 
        
        let x = -b_f64 / (2.0 * a_f64);
        let y = discrim_f64.abs().sqrt() / (2.0 * a_f64);

        NavigationFeatures {
            cos_x: (2.0 * PI * x).cos(),
            sin_x: (2.0 * PI * x).sin(),
            log_y: y.ln(),
        }
    }
}

/// Projector (投影仪)
/// 负责将代数状态(Soul)实体化为逻辑(Body)。
/// 
/// [Architecture Change]: 实现了 "Split Projection" 模式。
/// - project_exact: 用于生成最终真理 (Truth)，具备雪崩效应。
/// - project_heuristic: 用于优化器引导 (Will)，具备平滑性。
pub struct Projector {
    pub p_base: u64, 
}

impl Projector {
    pub fn new(p_base: u64) -> Self {
        Self { p_base }
    }

    /// [Truth Layer] 精确投影
    /// 
    /// 直接哈希代数结构的规范形式 (Canonical Form)。
    /// 任何微小的代数结构变化都会导致输出的剧烈跳变 (Avalanche Effect)。
    /// 这是用于 STP 验证和最终逻辑生成的唯一标准。
    pub fn project_exact(&self, state: &IdealClass, time: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        
        // 1. Structural Hashing (对 (a,b,c) 进行强绑定)
        // 只有完全相同的代数元素才会产生相同的哈希
        state.a.hash(&mut hasher);
        state.b.hash(&mut hasher);
        state.c.hash(&mut hasher);
        
        // 2. Time entanglement
        time.hash(&mut hasher);
        
        let hash = hasher.finish();
        hash % self.p_base
    }

    /// [Will Layer] 启发式投影
    /// 
    /// 基于模形式几何特征的分桶映射。
    /// 具有局部平滑性 (Lipschitz)，相邻的代数状态大概率落在相同或相邻的桶中。
    /// 用于计算 "Residual Energy" 以引导优化器。
    pub fn project_heuristic(&self, state: &IdealClass, time: u64) -> u64 {
        // 1. Extract Smooth Features
        let features = NavigationFeatures::extract(state);
        
        // 2. Discretize (Bucketing)
        const GRID_RESOLUTION: f64 = 1000.0;
        let bucket_cos = (features.cos_x * GRID_RESOLUTION) as i64;
        let bucket_sin = (features.sin_x * GRID_RESOLUTION) as i64;
        let bucket_log_y = (features.log_y * GRID_RESOLUTION) as i64;
        
        // 3. Hash the Bucket (Not the exact state)
        let mut hasher = DefaultHasher::new();
        bucket_cos.hash(&mut hasher);
        bucket_sin.hash(&mut hasher);
        bucket_log_y.hash(&mut hasher);
        time.hash(&mut hasher);
        
        let hash = hasher.finish();
        hash % self.p_base
    }

    /// 序列实体化 (默认为精确模式)
    /// 用于生成最终提交给用户的 Proof Path。
    pub fn project_sequence(&self, seed: &IdealClass, depth: usize) -> Vec<u64> {
        let mut current_state = seed.clone();
        let mut path = Vec::with_capacity(depth);

        for layer in 0..depth {
            // 使用 Exact Projection 保证真理的唯一性
            let digit = self.project_exact(&current_state, layer as u64);
            path.push(digit);
            current_state = current_state.square();
        }

        path
    }
}
