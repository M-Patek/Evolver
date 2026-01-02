use crate::soul::algebra::IdealClass;
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use num_traits::ToPrimitive;
use num_bigint::BigInt;

/// NavigationFeatures (导航特征)
/// Represents the geometric invariants of an ideal class projected onto a smooth cylindrical manifold.
/// 
/// [Internal] 这个结构体现在是内部私有的，对外隐藏了几何提取的复杂性。
#[derive(Debug, Clone)]
struct NavigationFeatures {
    /// Cyclic embedding of the real part x = -b/2a
    pub cos_x: f64,
    pub sin_x: f64,
    
    /// Logarithmic scaling of the imaginary part y = sqrt(|D|)/2a
    pub log_y: f64,
}

impl NavigationFeatures {
    /// Extracts smooth geometric features from a rigorous algebraic state.
    /// 从代数状态中提取模形式几何特征 \tau = x + iy
    fn extract(state: &IdealClass) -> Self {
        // 1. Convert BigInt to f64 for geometric heuristic calculation.
        // Precision loss is acceptable for the "Will" (Heuristic).
        let a_f64 = state.a.to_f64().unwrap_or(f64::MAX); 
        let b_f64 = state.b.to_f64().unwrap_or(f64::MAX);
        
        // 计算判别式 Delta = b^2 - 4ac
        // 注意：IdealClass 本身不存储 Delta，需即时计算以保证一致性
        let four = BigInt::from(4);
        let b_sq = &state.b * &state.b;
        let ac4 = &four * &state.a * &state.c;
        let delta = b_sq - ac4;
        let discrim_f64 = delta.to_f64().unwrap_or(f64::MIN); 
        
        // 2. Calculate the modular point tau = x + iy
        // x = -b / 2a
        let x = -b_f64 / (2.0 * a_f64);
        
        // y = sqrt(|Delta|) / 2a
        let y = discrim_f64.abs().sqrt() / (2.0 * a_f64);

        // 3. Project to Feature Space
        NavigationFeatures {
            cos_x: (2.0 * PI * x).cos(),
            sin_x: (2.0 * PI * x).sin(),
            log_y: y.ln(),
        }
    }
}

/// Projector (投影仪)
/// The core component of the Body. It is responsible for materializing 
/// algebraic states (Soul) into discrete logical digits (Logic).
///
/// 它整合了之前的 Navigator, Topology 和 Decoder。
pub struct Projector {
    pub p_base: u64, // 投影基数 (通常是一个素数，如 409)
}

impl Projector {
    /// 创建一个新的投影仪
    pub fn new(p_base: u64) -> Self {
        Self { p_base }
    }

    /// 单步投影 (Single Step Projection)
    /// 将特定时间点/层级的代数状态投影为离散数字。
    /// 
    /// This implements the mapping: \Psi: Cl(\Delta) x Time -> \mathbb{Z}_p
    pub fn project(&self, state: &IdealClass, time: u64) -> u64 {
        // 1. Extract Smooth Features (The Navigation Layer)
        // 利用模形式几何消除 (a,b) 系数的跳变不连续性
        let features = NavigationFeatures::extract(state);
        
        // 2. Discretize (The Bucketing Layer)
        // 将平滑的几何流形映射到离散网格
        const GRID_RESOLUTION: f64 = 1000.0;
        
        let bucket_cos = (features.cos_x * GRID_RESOLUTION) as i64;
        let bucket_sin = (features.sin_x * GRID_RESOLUTION) as i64;
        let bucket_log_y = (features.log_y * GRID_RESOLUTION) as i64;
        
        // 3. Hash to Z_p (The Deterministic Layer)
        // 混合几何特征与时间索引，生成动态逻辑
        let mut hasher = DefaultHasher::new();
        bucket_cos.hash(&mut hasher);
        bucket_sin.hash(&mut hasher);
        bucket_log_y.hash(&mut hasher);
        time.hash(&mut hasher); // Logic evolves over time even if state is static
        
        let hash = hasher.finish();
        
        hash % self.p_base
    }

    /// 序列实体化 (Sequence Materialization)
    /// 将初始种子展开为完整的逻辑路径。
    /// 
    /// 逻辑：
    /// 1. 投影当前状态 -> Digit
    /// 2. 状态自旋演化 (Squaring): S_{t+1} = S_t^2
    /// 3. 重复直到达到指定深度
    pub fn project_sequence(&self, seed: &IdealClass, depth: usize) -> Vec<u64> {
        let mut current_state = seed.clone();
        let mut path = Vec::with_capacity(depth);

        for layer in 0..depth {
            // A. Projection
            let digit = self.project(&current_state, layer as u64);
            path.push(digit);

            // B. Evolution
            // 依赖 Phase 1 中给 IdealClass 添加的 square() 方法
            current_state = current_state.square();
        }

        path
    }
}
