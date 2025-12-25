// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::core::affine::AffineTuple;
use crate::phase3::core::primes::hash_to_prime;
use crate::phase3::topology::tensor::Coordinate; // 确保引用正确的 Coordinate 定义
use rug::Integer;
use std::collections::{HashMap, HashSet};
use blake3::Hasher;

/// [Optimization]: K-D Tree Node
/// 用于加速高维空间最近邻搜索的数据结构
#[derive(Debug)]
pub struct KdNode {
    pub point: Coordinate,
    pub left: Option<Box<KdNode>>,
    pub right: Option<Box<KdNode>>,
    pub axis: usize,
}

/// 🗺️ VocabularyTensor: 静态词汇宇宙 (The Atlas)
/// 存储了 Token 在超空间中的确切位置。
pub struct VocabularyTensor {
    /// 正向映射: Coordinate -> Token Prime
    pub star_map: HashMap<Coordinate, Integer>,
    /// 反向映射: Token Prime -> Token ID (用于最终解码)
    pub prime_to_id: HashMap<Integer, u32>,
    
    /// K-D Tree Root for O(log N) search
    pub kd_tree: Option<Box<KdNode>>,
    
    pub dimensions: usize,
    pub side_length: usize,
}

impl VocabularyTensor {
    pub fn new(vocab_size: u32, dimensions: usize, side_length: usize) -> Self {
        let mut star_map = HashMap::new();
        let mut prime_to_id = HashMap::new();
        let mut points_for_tree = Vec::new();
        
        let mut occupied_primes: HashSet<Integer> = HashSet::new();
        let l = side_length as u64;
        
        // 初始化宇宙：将所有 Token 映射到空间中
        // [Mapping Strategy]: 
        // Token 被放置在固定的“家”中 (Static Addressing)。
        // 模型的任务是演化状态 S，使得 Hash(S) 精确指向这个家。
        for tid in 0..vocab_size {
            // 1. 计算确定性坐标 (Linear Layout)
            let mut coord = Vec::with_capacity(dimensions);
            let mut temp = tid as u64;
            for _ in 0..dimensions {
                coord.push((temp % l) as usize);
                temp /= l;
            }

            // 2. [DCAP Algorithm]: 生成绝对唯一的 Token Prime
            let base_token_str = format!("tok_{}", tid);
            let p = Self::generate_unique_prime(&base_token_str, &occupied_primes);
            
            occupied_primes.insert(p.clone());
            star_map.insert(coord.clone(), p.clone());
            prime_to_id.insert(p, tid);
            points_for_tree.push(coord);
        }

        // 构建 K-D Tree
        let kd_tree = Self::build_kdtree(&mut points_for_tree, 0, dimensions);

        VocabularyTensor {
            star_map,
            prime_to_id,
            kd_tree,
            dimensions,
            side_length,
        }
    }

    fn generate_unique_prime(base_str: &str, occupied: &HashSet<Integer>) -> Integer {
        let mut nonce = 0u64;
        const MAX_COLLISION_RETRIES: u64 = 1_000_000;

        while nonce < MAX_COLLISION_RETRIES {
            let input_str = if nonce == 0 {
                base_str.to_string()
            } else {
                format!("{}#collision_fix_{}", base_str, nonce)
            };

            if let Ok(candidate) = hash_to_prime(&input_str, 64) {
                if !occupied.contains(&candidate) {
                    return candidate;
                }
            }
            nonce += 1;
        }
        panic!("❌ Fatal Error: Vocabulary Space Exhausted.");
    }

    fn build_kdtree(points: &mut [Coordinate], depth: usize, k: usize) -> Option<Box<KdNode>> {
        if points.is_empty() { return None; }

        let axis = depth % k;
        points.sort_by(|a, b| a[axis].cmp(&b[axis]));
        let mid = points.len() / 2;

        let point = points[mid].clone();
        let (left_slice, right_slice_inclusive) = points.split_at_mut(mid);
        let (_, right_slice) = right_slice_inclusive.split_first_mut().unwrap();

        Some(Box::new(KdNode {
            point,
            left: Self::build_kdtree(left_slice, depth + 1, k),
            right: Self::build_kdtree(right_slice, depth + 1, k),
            axis,
        }))
    }
}

/// [NEW STRUCT]: 解码结果
pub struct DecodeResult {
    pub token_id: u32,
    pub drift: usize, // 曼哈顿漂移量
}

/// 🧭 InverseDecoder: 坐标导航器 (Phase 4 Upgraded)
pub struct InverseDecoder {
    pub vocab_tensor: VocabularyTensor,
    /// 动态搜索半径：如果直接找不到，允许在多大范围内搜索
    pub search_radius: usize,
}

impl InverseDecoder {
    pub fn new(vocab_size: u32) -> Self {
        InverseDecoder {
            vocab_tensor: VocabularyTensor::new(vocab_size, 4, 32),
            search_radius: 5, // 默认允许一定的模糊导航
        }
    }

    /// 📍 Decode: S_state -> Coordinate -> Nearest Token
    /// 
    /// [Phase 4 Core Logic]:
    /// 输入是 (1, S)。我们将 S 视为包含丰富语义的高维实体，
    /// 通过哈希投影将其映射到坐标系中。
    pub fn decode(&self, target_root: &AffineTuple) -> Result<DecodeResult, String> {
        // 1. Extract Coordinate via Semantic Hashing
        let predicted_coord = self.extract_coordinate(target_root);

        // 2. Exact Match Check (Zero Drift)
        if let Some(token_prime) = self.vocab_tensor.star_map.get(&predicted_coord) {
             if let Some(&tid) = self.vocab_tensor.prime_to_id.get(token_prime) {
                 return Ok(DecodeResult { token_id: tid, drift: 0 });
             }
        }

        // 3. Robust KNN Search (Non-Zero Drift)
        if let Some(nearest_coord) = self.find_nearest_neighbor_robust(&predicted_coord) {
            let token_prime = self.vocab_tensor.star_map.get(&nearest_coord).unwrap();
            let tid = self.vocab_tensor.prime_to_id.get(token_prime).unwrap();
            let drift = self.manhattan_distance(&predicted_coord, &nearest_coord);
            
            return Ok(DecodeResult { token_id: *tid, drift });
        }

        Err("❌ Navigation Lost: Entropy too high, no nearby stars found within horizon.".to_string())
    }

    /// 🌀 [CORE REWRITE]: Semantic Coordinate Extraction
    /// 
    /// 旧逻辑使用了 `p_factor`，在 Phase 2 之后 P 恒为 1，会导致坐标塌缩。
    /// 新逻辑：Coordinate = Hash(Serialize(S) || Domain_Tag)
    fn extract_coordinate(&self, tuple: &AffineTuple) -> Coordinate {
        let s = &tuple.q_shift; // ClassGroupElement S

        let mut hasher = Hasher::new();
        // Domain Separation Tag for Phase 4
        hasher.update(b"HTP_NAVIGATION_V4_COORDINATE");
        
        // Serialize S components (a, b, c)
        // 确保序列化顺序严格确定
        hasher.update(&s.a.to_digits(rug::integer::Order::Lsf));
        hasher.update(&s.b.to_digits(rug::integer::Order::Lsf));
        hasher.update(&s.c.to_digits(rug::integer::Order::Lsf));
        
        let hash = hasher.finalize();
        
        // Map Hash to Coordinate Dimensions
        let mut coord = Vec::new();
        let l = self.vocab_tensor.side_length as u64;
        let dim = self.vocab_tensor.dimensions;
        
        // 从 Hash 中提取足够多的熵
        let bytes = hash.as_bytes();
        // 取前 8 字节作为种子整数 (u64 足够覆盖大多数 Tensor 尺寸)
        let mut val = u64::from_le_bytes(bytes[0..8].try_into().unwrap());

        for _ in 0..dim {
            coord.push((val % l) as usize);
            val /= l;
        }
        coord
    }

    /// 🔎 [Robust] K-D Tree Search
    /// 增加了搜索半径约束，防止返回毫不相关的结果
    fn find_nearest_neighbor_robust(&self, target: &Coordinate) -> Option<Coordinate> {
        let mut best_dist = usize::MAX;
        let mut best_coord = None;

        if let Some(ref root) = self.vocab_tensor.kd_tree {
            self.search_kdtree_recursive(root, target, &mut best_dist, &mut best_coord);
        }
        
        // [Constraint]: 如果最近邻也太远，说明模型完全迷失了方向 (Hallucination Risk)
        if best_dist > self.search_radius {
            return None;
        }

        best_coord
    }

    fn search_kdtree_recursive(
        &self, 
        node: &KdNode, 
        target: &Coordinate, 
        best_dist: &mut usize, 
        best_coord: &mut Option<Coordinate>
    ) {
        let d = self.manhattan_distance(&node.point, target);
        if d < *best_dist {
            *best_dist = d;
            *best_coord = Some(node.point.clone());
        }

        if *best_dist == 0 { return; }

        let axis = node.axis;
        let diff = (target[axis] as isize) - (node.point[axis] as isize);
        
        let (near, far) = if diff <= 0 {
            (&node.left, &node.right)
        } else {
            (&node.right, &node.left)
        };

        if let Some(ref child) = near {
            self.search_kdtree_recursive(child, target, best_dist, best_coord);
        }

        let axis_dist = diff.abs() as usize;
        if axis_dist < *best_dist {
            if let Some(ref child) = far {
                self.search_kdtree_recursive(child, target, best_dist, best_coord);
            }
        }
    }

    fn manhattan_distance(&self, a: &Coordinate, b: &Coordinate) -> usize {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| if x > y { x - y } else { y - x })
            .sum()
    }
}
