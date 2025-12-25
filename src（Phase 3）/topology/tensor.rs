// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::collections::{HashMap, BTreeMap};
use rug::Integer;
use crate::core::affine::AffineTuple;
use blake3;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

// [CONFIG]: 安全性硬限制
// 即使在极端内存压力下，也不允许单点历史无限膨胀
const MAX_TIMELINE_DEPTH: usize = 64; 
// 全局容量软上限 (Soft Limit)
const GLOBAL_CAPACITY_LIMIT: usize = 10_000_000;
// 每次驱逐的批次大小，避免频繁触发
const EVICTION_BATCH_SIZE: usize = 100;

pub type Coordinate = Vec<usize>;

/// [Theoretical Best]: 微观时间线容器
/// 当空间发生碰撞时，我们在时间维度上展开，保证逻辑的因果完备性。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MicroTimeline {
    /// Key: Timestamp (Logic Sequence), Value: Affine Event
    /// BTreeMap 保证了按时间戳严格排序，这对于非交换代数至关重要。
    pub events: BTreeMap<u64, AffineTuple>,
}

impl MicroTimeline {
    pub fn new() -> Self {
        MicroTimeline {
            events: BTreeMap::new(),
        }
    }

    /// [DoS Protection]: 限制单点历史深度
    /// 如果一个坐标积累了过多的历史事件（可能是攻击者在刷热点），
    /// 我们必须修剪最旧的事件以释放内存。
    pub fn prune(&mut self) {
        if self.events.len() > MAX_TIMELINE_DEPTH {
            // 保留最新的 N 个，移除旧的
            // 这是一个 O(K) 操作，比无限增长安全得多
            let split_point = self.events.len().saturating_sub(MAX_TIMELINE_DEPTH);
            // 找到需要保留的第一个 key
            if let Some(&first_keep_key) = self.events.keys().nth(split_point) {
                // split_off 返回 >= key 的部分（即新的部分），我们将旧的部分丢弃
                let keep = self.events.split_off(&first_keep_key);
                self.events = keep;
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HyperTensor {
    pub dimensions: usize,
    pub side_length: usize,
    pub discriminant: Integer,
    
    /// [Upgrade]: Data 无论是空间还是时间，都是正交的
    /// HashMap<Space, BTreeMap<Time, Event>>
    /// 注意：为了真正的并发性能，未来建议升级为 DashMap 或分片锁结构。
    pub data: HashMap<Coordinate, MicroTimeline>,
    
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
        hasher.update(b":htp:coord:v3:orthogonal"); // Version Bump
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

    /// [FIXED]: 弹性插入 (Resilient Insertion)
    /// 解决了 DoS 漏洞：当容量满时，不再报错拒绝服务，而是执行随机驱逐 (Random Eviction)。
    /// 这保证了系统在攻击下的可用性 (Availability)。
    pub fn insert(&mut self, user_id: &str, new_tuple: AffineTuple, timestamp: u64) -> Result<(), String> {
        // [DoS Defense 1]: 全局容量检查与紧急驱逐
        if self.data.len() >= GLOBAL_CAPACITY_LIMIT {
            self.perform_emergency_eviction();
        }

        let coord = self.map_id_to_coord_hash(user_id);
        
        // 获取或创建微观时间线
        let timeline = self.data.entry(coord).or_insert_with(MicroTimeline::new);
        
        // [DoS Defense 2]: 单点深度修剪
        // 防止攻击者盯着一个坐标无限写入
        timeline.prune();
        
        timeline.events.insert(timestamp, new_tuple);

        self.cached_root = None;
        Ok(())
    }

    /// 🧹 紧急驱逐策略 (Emergency Eviction Strategy)
    /// 当系统过载时，随机丢弃一部分数据以腾出空间。
    /// 相比于 LRU，随机驱逐在 HashMap 上是 O(1) 的，更适合抗 DoS。
    fn perform_emergency_eviction(&mut self) {
        // 由于 Rust HashMap 的迭代顺序是不确定的（基于 Hash 种子），
        // 直接取 iter().next() 就等同于伪随机选择。
        // 我们批量移除 key 以减少 rehashing 开销。
        
        let keys_to_remove: Vec<Coordinate> = self.data.keys()
            .take(EVICTION_BATCH_SIZE)
            .cloned()
            .collect();

        for k in keys_to_remove {
            self.data.remove(&k);
        }
        
        // log::warn!("⚠️ HyperTensor Capacity Limit Reached. Evicted {} entries.", EVICTION_BATCH_SIZE);
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

    // 辅助：获取某个坐标的聚合状态（用于 Proof 生成等）
    pub fn get_collapsed_state(&self, coord: &Coordinate) -> Result<AffineTuple, String> {
        if let Some(timeline) = self.data.get(coord) {
            let mut agg = AffineTuple::identity(&self.discriminant);
            for tuple in timeline.events.values() {
                agg = agg.compose(tuple, &self.discriminant)?;
            }
            Ok(agg)
        } else {
            Ok(AffineTuple::identity(&self.discriminant))
        }
    }

    pub fn get_segment_tree_path(&self, coord: &Coordinate, _axis: usize) -> Vec<AffineTuple> {
        let mut path = Vec::new();
        // 这里需要返回聚合后的状态作为叶子节点
        if let Ok(t) = self.get_collapsed_state(coord) {
            path.push(t);
        } else {
            // Error fallback
            path.push(AffineTuple::identity(&self.discriminant));
        }
        
        if self.side_length > 1 {
             path.push(AffineTuple::identity(&self.discriminant));
        }
        path
    }
}
