use crate::soul::algebra::ClassGroupElement;
use crate::body::navigator::NavigationFeatures;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Clone)]
pub struct VPuNNConfig {
    pub depth: usize,
    pub p_base: u64,
    pub layer_decay: f64,
}

/// Projects a continuous algebraic state into a discrete logical symbol sequence.
/// 
/// Method: State -> Smooth Features -> Grid Discretization (Bucketing) -> Hash -> Z_p
pub fn project_state_to_digits(state: &ClassGroupElement, config: &VPuNNConfig, sequence_index: u64) -> u64 {
    // 1. Extract Smooth Features (The Navigation Layer)
    let features = NavigationFeatures::extract(state);
    
    // 2. Discretize (The Bucketing Layer)
    // We map the continuous features to a discrete grid.
    // Resolution determines the "sensitivity" of the logic to algebraic changes.
    const GRID_RESOLUTION: f64 = 1000.0;
    
    // x is periodic [-1, 1], simply scale and cast
    let bucket_cos = (features.cos_x * GRID_RESOLUTION) as i64;
    let bucket_sin = (features.sin_x * GRID_RESOLUTION) as i64;
    
    // y is logarithmic, valid range typically within feasible float bounds
    let bucket_log_y = (features.log_y * GRID_RESOLUTION) as i64;
    
    // 3. Hash to Z_p (The Deterministic Layer)
    // We mix the bucket coordinates with the sequence index to generate dynamic logic
    let mut hasher = DefaultHasher::new();
    bucket_cos.hash(&mut hasher);
    bucket_sin.hash(&mut hasher);
    bucket_log_y.hash(&mut hasher);
    sequence_index.hash(&mut hasher); // Logic evolves over time even if state is static
    
    let hash = hasher.finish();
    
    hash % config.p_base
}
