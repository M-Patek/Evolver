// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::collections::HashMap;
use rug::Integer;
use crate::core::affine::AffineTuple;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub type Coordinate = Vec<usize>;

/// 🌳 TimeSegmentTree: 微观历史树
/// 解决 "Merge on Collision" 导致的不可验证问题。
/// 它不再粗暴地融合数据，而是维护一个有序的时间线结构。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimeSegmentTree {
    /// 原始事件序列 (Leaves)
    /// 在生产环境中，这应该是一个 Merkle Mountain Range (MMR) 以节省空间，
    /// 但为了逻辑演示，我们保留完整序列以支持 Witness 生成。
    pub leaves: Vec<AffineTuple>,
}

impl TimeSegmentTree {
    pub fn new() -> Self {
        TimeSegmentTree { leaves: Vec::new() }
    }

    /// 📝 Append: 添加新事件（保持时间顺序）
    pub fn append(&mut self, tuple: AffineTuple) {
        self.leaves.push(tuple);
    }

    /// 🌲 Calculate Root: 计算当前单元的时间聚合根
    /// 使用非交换算子 ⊕_time (compose)
    /// Root = Leaf_0 ⊕ Leaf_1 ⊕ ... ⊕ Leaf_N
    pub fn root(&self, discriminant: &Integer) -> Result<AffineTuple, String> {
        if self.leaves.is_empty() {
            return Ok(AffineTuple::identity(discriminant));
        }

        // 使用 Segment Tree 方式两两聚合 (Balanced Fold)
        // 相比线性聚合，树状聚合能提供 O(log N) 的证明大小
        self.build_tree_recursive(&self.leaves, discriminant)
    }

    fn build_tree_recursive(&self, nodes: &[AffineTuple], discriminant: &Integer) -> Result<AffineTuple, String> {
        if nodes.len() == 0 {
            return Ok(AffineTuple::identity(discriminant));
        }
        if nodes.len() == 1 {
            return Ok(nodes[0].clone());
        }

        let mid = nodes.len() / 2;
        let left = self.build_tree_recursive(&nodes[0..mid], discriminant)?;
        let right = self.build_tree_recursive(&nodes[mid..], discriminant)?;

        // [Non-Commutative]: Left ⊕ Right
        left.compose(&right, discriminant)
    }

    /// 🔍 Generate Witness: 为指定索引的事件生成存在性证明
    /// 返回值: (Sibling Value, Is_Left_Sibling) 的列表
    pub fn generate_witness(&self, index: usize, discriminant: &Integer) -> Result<Vec<(AffineTuple, bool)>, String> {
        if index >= self.leaves.len() {
            return Err("Index out of bounds".to_string());
        }
        let mut witness = Vec::new();
        self.generate_witness_recursive(&self.leaves, index, 0, discriminant, &mut witness)?;
        Ok(witness)
    }

    fn generate_witness_recursive(
        &self, 
        nodes: &[AffineTuple], 
        target_abs_index: usize, 
        current_offset: usize,
        discriminant: &Integer,
        witness: &mut Vec<(AffineTuple, bool)>
    ) -> Result<AffineTuple, String> {
        if nodes.len() == 1 {
            return Ok(nodes[0].clone());
        }

        let mid = nodes.len() / 2;
        let left_slice = &nodes[0..mid];
        let right_slice = &nodes[mid..];

        // 判断目标在左子树还是右子树
        if target_abs_index < current_offset + mid {
            // Target inside Left
            let right_agg = self.build_tree_recursive(right_slice, discriminant)?;
            // 记录：我的右边有一个兄弟 (Right Sibling)
            // 在验证时，Proof = Me ⊕ Right
            witness.push((right_agg, false)); 
            
            let left_agg = self.generate_witness_recursive(left_slice, target_abs_index, current_offset, discriminant, witness)?;
            return left_agg.compose(&self.build_tree_recursive(right_slice, discriminant)?, discriminant);
        } else {
            // Target inside Right
            let left_agg = self.build_tree_recursive(left_slice, discriminant)?;
            // 记录：我的左边有一个兄弟 (Left Sibling)
            // 在验证时，Proof = Left ⊕ Me
            witness.push((left_agg, true));

            let right_agg = self.generate_witness_recursive(right_slice, target_abs_index, current_offset + mid, discriminant, witness)?;
            return left_agg.compose(&right_agg, discriminant);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HyperTensor {
    pub dimensions: usize,
    pub side_length: usize,
    pub discriminant: Integer,
    
    // [FIX]: Value 从单一的 AffineTuple 升级为 TimeSegmentTree
    // 这使得每个坐标点都能容纳无限的历史，并支持 Witness 提取。
    pub data: HashMap<Coordinate, TimeSegmentTree>,
    
    #[serde(skip)]
    pub cached_root: Option<AffineTuple>, 
}

impl HyperTensor {
    pub fn new(dim: usize, len: usize, discriminant: Integer) -> Self {
        HyperTensor {
            dimensions: dim,
            side_length: len,
            discriminant,
            data: HashMap::new(),
            cached_root: None,
        }
    }

    pub fn map_id_to_coord(&self, numeric_id: u64) -> Coordinate {
        let mut coord = Vec::with_capacity(self.dimensions);
        let mut temp = numeric_id;
        let l = self.side_length as u64;
        for _ in 0..self.dimensions {
            coord.push((temp % l) as usize);
            temp /= l;
        }
        coord
    }
    
    pub fn map_id_to_coord_hash(&self, user_id: &str) -> Coordinate {
        let mut hasher = blake3::Hasher::new();
        hasher.update(user_id.as_bytes());
        hasher.update(b":htp:coord:v2");
        let hash_output = hasher.finalize();
        
        let mut coord = Vec::with_capacity(self.dimensions);
        let reader = hash_output.as_bytes();
        let l = self.side_length as u128;
        
        let mut val = u128::from_le_bytes(reader[0..16].try_into().unwrap());
        
        for _ in 0..self.dimensions {
            coord.push((val % l) as usize);
            val /= l;
        }
        coord
    }

    // [FIX]: 现在的 Insert 不再是破坏性的 Merge，而是结构化的 Append
    pub fn insert(&mut self, user_id: &str, new_tuple: AffineTuple) -> Result<(), String> {
        let coord = self.map_id_to_coord_hash(user_id);
        
        self.data.entry(coord)
            .or_insert_with(TimeSegmentTree::new)
            .append(new_tuple);

        self.cached_root = None;
        Ok(())
    }
    
    pub fn save_to_disk(&self, path: &str) -> Result<(), String> {
        let file = File::create(path).map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_from_disk(path: &str) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let tensor: HyperTensor = bincode::deserialize_from(reader).map_err(|e| e.to_string())?;
        Ok(tensor)
    }

    // 获取路径证明 (简化版 API)
    pub fn get_segment_tree_path(&self, coord: &Coordinate, _axis: usize) -> Vec<AffineTuple> {
        // 在 Phase 2 中，这里的语义稍微有些混杂
        // 如果是获取 "Cell 的聚合证明"，应该调用 tree.root()
        // 如果是获取 "Cell 内部的时间证明"，应该调用 tree.generate_witness()
        // 这里返回 Root 作为占位，代表该坐标的整体状态
        if let Some(tree) = self.data.get(coord) {
            if let Ok(root) = tree.root(&self.discriminant) {
                return vec![root];
            }
        }
        vec![AffineTuple::identity(&self.discriminant)]
    }
    
    pub fn get(&self, coord: &Coordinate) -> AffineTuple {
        match self.data.get(coord) {
            Some(tree) => tree.root(&self.discriminant).unwrap_or(AffineTuple::identity(&self.discriminant)),
            None => AffineTuple::identity(&self.discriminant),
        }
    }
}
