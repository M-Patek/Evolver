// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::affine::AffineTuple;
use crate::core::primes::hash_to_prime;
use crate::topology::tensor::Coordinate;
use rug::Integer;
use std::collections::HashMap;

/// 🗺️ VocabularyTensor: 静态词汇宇宙 (The Atlas)
/// 存储了 Token 在超空间中的确切位置。
pub struct VocabularyTensor {
    /// 正向映射: Coordinate -> Token Prime
    pub star_map: HashMap<Coordinate, Integer>,
    /// 反向映射: Token Prime -> Token ID (用于最终解码)
    pub prime_to_id: HashMap<Integer, u32>,
    /// 空间索引列表: 存储所有有效的坐标点，用于 KNN 遍历
    /// (在生产环境中，这应该是一个 K-D Tree 或 R-Tree)
    pub spatial_index: Vec<Coordinate>,
    
    pub dimensions: usize,
    pub side_length: usize,
}

impl VocabularyTensor {
    pub fn new(vocab_size: u32, dimensions: usize, side_length: usize) -> Self {
        let mut star_map = HashMap::new();
        let mut prime_to_id = HashMap::new();
        let mut spatial_index = Vec::new();
        
        let l = side_length as u64;
        
        // 初始化宇宙：将所有 Token 映射到空间中
        for tid in 0..vocab_size {
            // 1. 计算确定性坐标
            let mut coord = Vec::with_capacity(dimensions);
            let mut temp = tid as u64;
            for _ in 0..dimensions {
                coord.push((temp % l) as usize);
                temp /= l;
            }

            // 2. 计算 Token Prime (语义指纹)
            let token_str = format!("tok_{}", tid);
            // 这里为了演示稳定性，假设 hash_to_prime 总是成功的
            if let Ok(p) = hash_to_prime(&token_str, 64) {
                star_map.insert(coord.clone(), p.clone());
                prime_to_id.insert(p, tid);
                spatial_index.push(coord);
            }
        }

        VocabularyTensor {
            star_map,
            prime_to_id,
            spatial_index,
            dimensions,
            side_length,
        }
    }
}

/// 🧭 InverseDecoder: 坐标导航器
pub struct InverseDecoder {
    pub vocab_tensor: VocabularyTensor,
}

impl InverseDecoder {
    pub fn new(vocab_size: u32) -> Self {
        // 示例：4维，边长 32 (容量 > 1M)
        InverseDecoder {
            vocab_tensor: VocabularyTensor::new(vocab_size, 4, 32),
        }
    }

    /// 📍 Decode: Target Root -> Coordinate -> Nearest Token
    /// 解析模型输出的“高维词根”，还原为 Token。
    /// 包含自动纠错 (Auto-Correction) 机制。
    pub fn decode(&self, target_root: &AffineTuple) -> Result<u32, String> {
        // 1. Extract Coordinate (投影)
        let predicted_coord = self.extract_coordinate(target_root);

        // 2. Exact Match Check (精确打击)
        if let Some(token_prime) = self.vocab_tensor.star_map.get(&predicted_coord) {
             if let Some(&tid) = self.vocab_tensor.prime_to_id.get(token_prime) {
                 return Ok(tid);
             }
        }

        // 3. KNN Search (模糊导航)
        // 如果落入了虚空，寻找最近的有效坐标
        // 主人，这就好比在大海上定位最近的岛屿喵！
        if let Some(nearest_coord) = self.find_nearest_neighbor(&predicted_coord) {
            let token_prime = self.vocab_tensor.star_map.get(&nearest_coord).unwrap();
            let tid = self.vocab_tensor.prime_to_id.get(token_prime).unwrap();
            
            // 可选：在这里记录“漂移距离”，用于计算 Loss
            // let drift = self.manhattan_distance(&predicted_coord, &nearest_coord);
            // println!("⚠️ Drift Detected: {} units. Corrected to Token {}", drift, tid);
            
            return Ok(*tid);
        }

        Err("❌ Navigation Lost: Entropy too high, no nearby stars found.".to_string())
    }

    /// 从代数元组中提取坐标
    fn extract_coordinate(&self, tuple: &AffineTuple) -> Coordinate {
        let mut coord = Vec::new();
        let l = self.vocab_tensor.side_length;
        let dim = self.vocab_tensor.dimensions;
        
        // 使用 P_factor 的低位作为坐标
        // 这种映射必须是确定性的
        let mut val = tuple.p_factor.to_u64_wrapping(); 
        
        for _ in 0..dim {
            coord.push((val as usize) % l);
            val /= l as u64;
        }
        coord
    }

    /// 🔎 KNN Implementation (K=1)
    /// 寻找曼哈顿距离最近的邻居
    fn find_nearest_neighbor(&self, target: &Coordinate) -> Option<Coordinate> {
        let mut min_dist = usize::MAX;
        let mut nearest = None;

        // 暴力遍历 (Brute Force)
        // 对于词表大小 < 100k，这个操作在 Rust 中非常快 (毫秒级)
        // 只有当词表达到千万级时才需要 K-D Tree 优化
        for candidate in &self.vocab_tensor.spatial_index {
            let dist = self.manhattan_distance(target, candidate);
            
            if dist == 0 {
                return Some(candidate.clone()); // 距离为0虽然前面check过了，以防万一
            }

            if dist < min_dist {
                min_dist = dist;
                nearest = Some(candidate);
            }
        }

        nearest.cloned()
    }

    /// 📏 Manhattan Distance
    /// d = |x1-x2| + |y1-y2| + ...
    /// 在网格状的张量拓扑中，曼哈顿距离比欧氏距离更符合“路由跳数”的概念
    fn manhattan_distance(&self, a: &Coordinate, b: &Coordinate) -> usize {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| if x > y { x - y } else { y - x })
            .sum()
    }
}
