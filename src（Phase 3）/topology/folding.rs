// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::tensor::HyperTensor;
use crate::phase3::core::affine::AffineTuple;
use crate::phase3::core::algebra::ClassGroupElement;
use rug::Integer;
use std::collections::HashMap;

impl HyperTensor {
    // [API CHANGE]: 公开的计算入口，默认使用自然序 [0, 1, 2, ...]
    pub fn calculate_global_root(&mut self) -> Result<AffineTuple, String> {
        // 构建自然序: [0, 1, 2, ... D-1]
        let default_order: Vec<usize> = (0..self.dimensions).collect();
        
        // 注意：这里的 cached_root 应当基于新的折叠逻辑失效时清除
        if let Some(ref root) = self.cached_root {
             // return Ok(root.clone()); // 暂时禁用缓存以确保维度置换测试的正确性
        }

        let root = self.compute_root_internal(&default_order)?;
        // self.cached_root = Some(root.clone());
        Ok(root)
    }

    // [API CHANGE]: 内部计算现在支持“维度置换”
    pub fn compute_root_internal(&self, dim_order: &[usize]) -> Result<AffineTuple, String> {
        // [Phase 1]: Micro-Fold (Time Aggregation - Non-Commutative)
        let flat_data = self.reconstruct_spatial_snapshot()?;

        // [Phase 2]: Macro-Fold (Spatial Aggregation - Commutative)
        // 从深度 0 开始递归，依照 dim_order 指定的顺序
        let root = self.fold_sparse(0, dim_order, &flat_data)?;
        Ok(root)
    }

    /// 🛠️ 从时间线重建空间快照
    fn reconstruct_spatial_snapshot(&self) -> Result<HashMap<Vec<usize>, AffineTuple>, String> {
        let mut snapshot = HashMap::new();
        let one = Integer::from(1);
        let identity_q = ClassGroupElement::identity(&self.discriminant);

        for (coord, time_tree) in &self.data {
            // [Time Collapse]: 这一步体现了因果律 (非交换)
            let cell_time_root = time_tree.root(&self.discriminant)?;

            // [Sparse Optimization]
            if cell_time_root.p_factor != one {
                snapshot.insert(coord.clone(), cell_time_root);
            } else {
                if cell_time_root.q_shift != identity_q {
                     snapshot.insert(coord.clone(), cell_time_root);
                }
            }
        }
        Ok(snapshot)
    }

    // 核心算法：支持维度置换的稀疏折叠
    fn fold_sparse(
        &self,
        depth: usize, // 当前递归深度 (0..D)
        dim_order: &[usize], // 维度折叠顺序
        relevant_data: &HashMap<Vec<usize>, AffineTuple>
    ) -> Result<AffineTuple, String> {
        if relevant_data.is_empty() {
             return Ok(AffineTuple::identity(&self.discriminant));
        }

        if depth == self.dimensions {
             return Ok(AffineTuple::identity(&self.discriminant));
        }

        // [CRITICAL CHANGE]: 获取当前层需要折叠的“物理维度”
        // 这允许了 Fold(X->Y) 和 Fold(Y->X) 的自由切换
        let target_dim = dim_order[depth];

        // Grouping: 按 target_dim 的坐标值分组
        let mut groups: HashMap<usize, HashMap<Vec<usize>, AffineTuple>> = HashMap::new();
        for (coord, tuple) in relevant_data {
            if target_dim >= coord.len() { continue; }
            let idx = coord[target_dim];
            groups.entry(idx)
                .or_insert_with(HashMap::new)
                .insert(coord.clone(), tuple.clone());
        }

        let mut layer_agg = AffineTuple::identity(&self.discriminant);
        let mut sorted_indices: Vec<usize> = groups.keys().cloned().collect();
        sorted_indices.sort(); 

        for idx in sorted_indices {
            let sub_map = groups.get(&idx).unwrap();
            
            // Recurse: 深度 +1
            let sub_result = self.fold_sparse(depth + 1, dim_order, sub_map)?;
            
            // [BOUNDARY CHECK]: 必须使用 commutative_merge
            // 只有阿贝尔群的聚合才能保证 Fold(Order_A) == Fold(Order_B)
            layer_agg = layer_agg.commutative_merge(&sub_result, &self.discriminant)?;
        }

        Ok(layer_agg)
    }
}
