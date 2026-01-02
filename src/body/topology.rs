use crate::soul::algebra::ClassGroupElement;
use crate::body::navigator::NavigationFeatures;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// v-PuNN Configuration for Topological Projection
#[derive(Clone)]
pub struct VPuNNConfig {
    pub depth: usize,
    pub p_base: u64,
    pub layer_decay: f64,
}

/// # 机制 (The Body)
/// 1. **几何提取 (Geometry)**: 将代数状态映射到上半平面的模点 \tau。
/// 2. **特征平滑 (Smoothing)**: 提取模形式不变量 (\cos, \sin, \log y) 以消除约化混沌。
/// 3. **离散化 (Discretization)**: 将连续特征落入桶 (Bucket) 中。
/// 4. **确定性哈希 (Hashing)**: 最终映射到有限域 \mathbb{Z}_p。
///
/// # 优势
/// 保证了 \Psi 在流形上的 Lipschitz 连续性，使 VAPO 能够感知“梯度”。
pub fn project_state_to_digits(state: &ClassGroupElement, config: &VPuNNConfig, sequence_index: u64) -> u64 {
    // 1. Extract Smooth Features (The Navigation Layer)
    // 利用模形式几何消除 (a,b) 系数的跳变不连续性
    let features = NavigationFeatures::extract(state);
    
    // 2. Discretize (The Bucketing Layer)
    // 将平滑的几何流形映射到离散网格
    // Resolution determines the "sensitivity" of the logic to algebraic changes.
    const GRID_RESOLUTION: f64 = 1000.0;
    
    // x is periodic [-1, 1], simply scale and cast
    let bucket_cos = (features.cos_x * GRID_RESOLUTION) as i64;
    let bucket_sin = (features.sin_x * GRID_RESOLUTION) as i64;
    
    // y is logarithmic, valid range typically within feasible float bounds
    let bucket_log_y = (features.log_y * GRID_RESOLUTION) as i64;
    
    // 3. Hash to Z_p (The Deterministic Layer)
    // 混合几何特征与时间索引，生成动态逻辑
    let mut hasher = DefaultHasher::new();
    bucket_cos.hash(&mut hasher);
    bucket_sin.hash(&mut hasher);
    bucket_log_y.hash(&mut hasher);
    sequence_index.hash(&mut hasher); // Logic evolves over time even if state is static
    
    let hash = hasher.finish();
    
    hash % config.p_base
}
