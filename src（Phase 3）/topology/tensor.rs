// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::Integer;
use crate::phase3::core::affine::AffineTuple;
use crate::phase3::topology::merkle::IncrementalMerkleTree;
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use blake3::Hasher;
use std::collections::HashMap;

// [CONFIG]: Log Policy
const HOT_LAYER_SIZE: usize = 1024;

/// 📜 LogEntry: 不可变的历史单元 (Micro-History)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub index: u64,
    pub checkpoint_hash: [u8; 32],
    pub op_snapshot: AffineTuple,
    pub timestamp: u64,
}

/// 🗄️ EventLog: 审计日志
#[derive(Serialize, Deserialize)]
pub struct EventLog {
    pub hot_layer: Vec<LogEntry>,
    // Merkle Tree 仅用于审计日志完整性，不再作为 Global Root 的来源
    pub commitment_tree: IncrementalMerkleTree,
    #[serde(skip)]
    pub cold_file_path: String,
}

impl EventLog {
    pub fn new(cold_path: String) -> Self {
        EventLog {
            hot_layer: Vec::new(),
            commitment_tree: IncrementalMerkleTree::new(),
            cold_file_path: cold_path,
        }
    }

    pub fn append(&mut self, entry: LogEntry) -> Result<(), String> {
        self.commitment_tree.append(entry.checkpoint_hash);
        // Persist to cold storage (omitted for brevity in fix)
        // self.persist_to_cold(&entry)?;
        if self.hot_layer.len() >= HOT_LAYER_SIZE {
            self.hot_layer.remove(0);
        }
        self.hot_layer.push(entry);
        Ok(())
    }
}

/// 🧊 HyperTensor (Space-Time Manifold)
/// [FIXED]: 实现了真正的代数折叠，而非哈希树包装。
#[derive(Serialize, Deserialize)]
pub struct HyperTensor {
    pub dimensions: usize,
    pub side_length: usize,
    pub discriminant: Integer,
    
    // [Track A]: Time (Audit History)
    pub event_log: EventLog,

    // [Track B]: Space (Active Holographic State)
    // 这是一个稀疏映射：Coordinate -> Current Algebraic State
    // 它是 Macro-Fold 的基础数据源。
    pub active_data: HashMap<Vec<usize>, AffineTuple>,
}

impl HyperTensor {
    pub fn new(dim: usize, len: usize, discriminant: Integer) -> Self {
        HyperTensor {
            dimensions: dim,
            side_length: len,
            discriminant,
            event_log: EventLog::new("/tmp/htp_event_log.bin".to_string()),
            active_data: HashMap::new(),
        }
    }

    /// 简单的 1D -> N-D 映射
    pub fn map_id_to_coord(&self, numeric_id: u64) -> Vec<usize> {
        let mut coord = Vec::with_capacity(self.dimensions);
        let mut temp = numeric_id;
        let l = self.side_length as u64;
        for _ in 0..self.dimensions {
            coord.push((temp % l) as usize);
            temp /= l;
        }
        coord
    }

    /// 🖊️ Insert (Space-Time Dual Write)
    /// 同时更新线性日志（时间）和稀疏张量状态（空间）。
    pub fn insert(&mut self, _key: &str, checkpoint: AffineTuple, timestamp: u64) -> Result<(), String> {
        // 1. [Time Axis]: Append to Log for auditability
        // 构造哈希以绑定状态 (Fix applied previously)
        let mut hasher = Hasher::new();
        hasher.update(b"HTP_LOG_ENTRY_V1"); 
        hasher.update(&checkpoint.p_factor.to_digits(rug::integer::Order::Lsf));
        hasher.update(&checkpoint.q_shift.a.to_digits(rug::integer::Order::Lsf));
        hasher.update(&checkpoint.q_shift.b.to_digits(rug::integer::Order::Lsf));
        hasher.update(&checkpoint.q_shift.c.to_digits(rug::integer::Order::Lsf));
        let hash = hasher.finalize().into();

        let entry = LogEntry {
            index: self.event_log.commitment_tree.leaf_count,
            checkpoint_hash: hash,
            op_snapshot: checkpoint.clone(),
            timestamp,
        };
        self.event_log.append(entry)?;

        // 2. [Space Axis]: Update Active State for Folding
        // 在 Phase 3 中，timestamp 通常对应序列位置 (seq)，这里将其映射为空间坐标
        let coord = self.map_id_to_coord(timestamp);
        
        // [CRITICAL]: 这里的语义是 "Snapshot Update"。
        // 神经元在 t 时刻的状态是该位置的最新状态。
        // 直接更新 active_data 对应的坐标点。
        self.active_data.insert(coord, checkpoint);

        Ok(())
    }

    /// 📐 Calculate Global Root (Algebraic Folding)
    /// [THEORY COMPLIANCE]: 使用 Commutative Space Operator (⊗) 进行全息折叠。
    /// 结果是一个真正的 AffineTuple，包含群结构信息。
    pub fn calculate_global_root(&self) -> Result<AffineTuple, String> {
        // 如果张量为空，返回单位元
        if self.active_data.is_empty() {
            return Ok(AffineTuple::identity(&self.discriminant));
        }

        // 启动递归折叠，从维度 0 开始
        self.fold_recursive(0, &self.active_data)
    }

    /// 递归稀疏折叠逻辑
    /// O(N_active * log(Dimensions))
    fn fold_recursive(
        &self, 
        current_dim: usize, 
        current_view: &HashMap<Vec<usize>, AffineTuple>
    ) -> Result<AffineTuple, String> {
        // Base Case: 如果视图为空
        if current_view.is_empty() {
            return Ok(AffineTuple::identity(&self.discriminant));
        }

        // Base Case: 维度耗尽 (叶子节点不应该走到这里，因为是 Sparse 遍历)
        if current_dim >= self.dimensions {
             // 理论上不应发生，除非 coord 长度不一致。
             // 取任意一个值（实际上 view 此时应该只有一个元素，且 key 为空 vec）
             if let Some(val) = current_view.values().next() {
                 return Ok(val.clone());
             }
             return Ok(AffineTuple::identity(&self.discriminant));
        }

        // 1. Grouping: 按当前维度的切片分组
        // 例如：dim=0 时，把所有 x=0 的归一组，x=1 的归一组...
        let mut slices: HashMap<usize, HashMap<Vec<usize>, AffineTuple>> = HashMap::new();
        
        for (coord, tuple) in current_view {
            // 安全检查
            if current_dim >= coord.len() { continue; }
            
            let idx = coord[current_dim];
            
            // 存入子 map 时，key 不需要变（或者是去掉这一维？为了简单，我们保留完整 coord，只在递归时看下一维）
            slices.entry(idx)
                .or_insert_with(HashMap::new)
                .insert(coord.clone(), tuple.clone());
        }

        // 2. Aggregation: 对每个切片进行递归折叠
        let mut aggregated_slices = Vec::new();
        // 必须排序索引以保证确定性！虽然 Space Operator 是交换的，但浮点误差或结构稳定性需要确定性
        let mut sorted_indices: Vec<usize> = slices.keys().cloned().collect();
        sorted_indices.sort();

        for idx in sorted_indices {
            let sub_view = slices.get(&idx).unwrap();
            let sub_root = self.fold_recursive(current_dim + 1, sub_view)?;
            aggregated_slices.push(sub_root);
        }

        // 3. Commutative Merge (The Space Operator ⊗)
        // 将所有切片的折叠结果聚合在一起
        let mut layer_root = AffineTuple::identity(&self.discriminant);
        
        for slice_root in aggregated_slices {
            // [MATHEMATICAL CORE]: 使用交换合并
            // Tuple_A ⊗ Tuple_B = (P_A * P_B, Q_A * Q_B)
            layer_root = layer_root.commutative_merge(&slice_root, &self.discriminant)?;
        }

        Ok(layer_root)
    }

    // 占位符：未来实现真正的 Proof Path 提取
    pub fn get_segment_tree_path(&self, _coord: &Vec<usize>, _axis: usize) -> Vec<AffineTuple> {
        vec![AffineTuple::identity(&self.discriminant)] 
    }
}
