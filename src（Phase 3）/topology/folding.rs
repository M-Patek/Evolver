// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::tensor::HyperTensor;
use crate::core::affine::AffineTuple;
use std::collections::HashMap;

impl HyperTensor {
    pub fn calculate_global_root(&mut self) -> Result<AffineTuple, String> {
        if let Some(ref root) = self.cached_root {
            return Ok(root.clone());
        }

        let root = self.compute_root_internal()?;
        self.cached_root = Some(root.clone());
        Ok(root)
    }

    pub fn compute_root_internal(&self) -> Result<AffineTuple, String> {
        // [Phase 1]: Micro-Fold (Time Aggregation)
        // 将 data 中的 MicroTimeline 坍缩为单一的 AffineTuple
        // 这相当于在每个空间点上，先跑完所有的历史因果链。
        let mut flat_data: HashMap<Vec<usize>, AffineTuple> = HashMap::new();
        
        for (coord, timeline) in &self.data {
            let mut local_root = AffineTuple::identity(&self.discriminant);
            
            // BTreeMap.values() 保证了按 key (timestamp) 升序迭代
            // 完美保留了 A -> B 的非交换顺序: A.compose(B)
            for tuple in timeline.events.values() {
                local_root = local_root.compose(tuple, &self.discriminant)?;
            }
            
            flat_data.insert(coord.clone(), local_root);
        }

        // [Phase 2]: Macro-Fold (Spatial Aggregation)
        // 使用坍缩后的快照进行标准的空间折叠
        let root = self.fold_sparse(0, &flat_data)?;
        Ok(root)
    }

    // 原始的稀疏折叠算法保持不变，它只负责处理空间维度
    // 此时传入的 relevant_data 已经是去除了时间维度的扁平快照
    fn fold_sparse(
        &self,
        current_dim: usize,
        relevant_data: &HashMap<Vec<usize>, AffineTuple>
    ) -> Result<AffineTuple, String> {
        if relevant_data.is_empty() {
             return Ok(AffineTuple::identity(&self.discriminant));
        }

        if current_dim == self.dimensions {
             return Ok(AffineTuple::identity(&self.discriminant));
        }

        // 按当前维度的索引分组
        let mut groups: HashMap<usize, HashMap<Vec<usize>, AffineTuple>> = HashMap::new();
        for (coord, tuple) in relevant_data {
            if current_dim >= coord.len() { continue; }
            let idx = coord[current_dim];
            groups.entry(idx)
                .or_insert_with(HashMap::new)
                .insert(coord.clone(), tuple.clone());
        }

        let mut layer_agg = AffineTuple::identity(&self.discriminant);
        let mut sorted_indices: Vec<usize> = groups.keys().cloned().collect();
        sorted_indices.sort(); 

        for idx in sorted_indices {
            let sub_map = groups.get(&idx).unwrap();
            let sub_result = self.fold_sparse(current_dim + 1, sub_map)?;
            layer_agg = layer_agg.compose(&sub_result, &self.discriminant)?;
        }

        Ok(layer_agg)
    }
}
