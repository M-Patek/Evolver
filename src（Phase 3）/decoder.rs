// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::affine::AffineTuple;
use crate::core::primes::hash_to_prime;
use crate::topology::tensor::Coordinate;
use rug::Integer;
use std::collections::HashMap;

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
    
    /// [Legacy Index]: 线性列表，保留用于调试或全量遍历
    pub spatial_index: Vec<Coordinate>,

    /// [PERFORMANCE FIX]: K-D Tree Root
    /// 替换原先的暴力遍历，提供 O(log N) 的查询能力
    pub kd_tree: Option<Box<KdNode>>,
    
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

        // [PERFORMANCE FIX]: 构建 K-D Tree
        // 在初始化阶段花费 O(N log N) 时间建立索引，换取推理时的 O(log N)
        let mut points_for_tree = spatial_index.clone();
        let kd_tree = Self::build_kdtree(&mut points_for_tree, 0, dimensions);

        VocabularyTensor {
            star_map,
            prime_to_id,
            spatial_index,
            kd_tree,
            dimensions,
            side_length,
        }
    }

    /// 递归构建平衡 K-D Tree
    fn build_kdtree(points: &mut [Coordinate], depth: usize, k: usize) -> Option<Box<KdNode>> {
        if points.is_empty() {
            return None;
        }

        let axis = depth % k;
        // 按当前轴排序，取中位数作为分割点
        points.sort_by(|a, b| a[axis].cmp(&b[axis]));
        let mid = points.len() / 2;

        // 这里使用了 split_at_mut 来分割切片，但这需要所有权处理
        // 简单起见，我们交换中位数到中间，并递归处理
        let point = points[mid].clone();
        
        // 分割数组：[0..mid] 为左子树，[mid+1..] 为右子树
        let (left_slice, right_slice_inclusive) = points.split_at_mut(mid);
        let (_, right_slice) = right_slice_inclusive.split_first_mut().unwrap(); // 跳过 mid 本身

        Some(Box::new(KdNode {
            point,
            left: Self::build_kdtree(left_slice, depth + 1, k),
            right: Self::build_kdtree(right_slice, depth + 1, k),
            axis,
        }))
    }
}

/// [NEW STRUCT]: 解码结果，包含漂移量
/// 用于量化生成的精确度
pub struct DecodeResult {
    pub token_id: u32,
    pub drift: usize, // 曼哈顿距离
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
    /// 包含自动纠错 (Auto-Correction) 机制，并报告漂移值。
    pub fn decode(&self, target_root: &AffineTuple) -> Result<DecodeResult, String> {
        // 1. Extract Coordinate (投影)
        let predicted_coord = self.extract_coordinate(target_root);

        // 2. Exact Match Check (精确打击 - Zero Drift)
        // 哈希表查找是 O(1)，最快路径
        if let Some(token_prime) = self.vocab_tensor.star_map.get(&predicted_coord) {
             if let Some(&tid) = self.vocab_tensor.prime_to_id.get(token_prime) {
                 return Ok(DecodeResult {
                     token_id: tid,
                     drift: 0, // 完美命中
                 });
             }
        }

        // 3. K-D Tree Search (快速空间导航 - Non-Zero Drift)
        // [PERFORMANCE FIX]: 从 O(N) 优化至 O(log N)
        if let Some(nearest_coord) = self.find_nearest_neighbor_optimized(&predicted_coord) {
            let token_prime = self.vocab_tensor.star_map.get(&nearest_coord).unwrap();
            let tid = self.vocab_tensor.prime_to_id.get(token_prime).unwrap();
            
            // 计算漂移距离 (Penalty Score)
            let drift = self.manhattan_distance(&predicted_coord, &nearest_coord);
            
            return Ok(DecodeResult {
                token_id: *tid,
                drift,
            });
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

    /// 🔎 [Optimized] K-D Tree Search
    /// 使用树结构进行剪枝搜索
    fn find_nearest_neighbor_optimized(&self, target: &Coordinate) -> Option<Coordinate> {
        let mut best_dist = usize::MAX;
        let mut best_coord = None;

        if let Some(ref root) = self.vocab_tensor.kd_tree {
            self.search_kdtree_recursive(root, target, &mut best_dist, &mut best_coord);
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
        // 1. 计算当前节点距离
        let d = self.manhattan_distance(&node.point, target);
        if d < *best_dist {
            *best_dist = d;
            *best_coord = Some(node.point.clone());
        }

        // 如果距离为0，已是最优，无需继续
        if *best_dist == 0 { return; }

        // 2. 决定搜索顺序 (启发式：先搜目标点所在的那一侧)
        let axis = node.axis;
        let diff = (target[axis] as isize) - (node.point[axis] as isize);
        
        let (near, far) = if diff <= 0 {
            (&node.left, &node.right)
        } else {
            (&node.right, &node.left)
        };

        // 3. 递归搜索“近”侧
        if let Some(ref child) = near {
            self.search_kdtree_recursive(child, target, best_dist, best_coord);
        }

        // 4. 剪枝判断：是否需要搜索“远”侧？
        // 对于曼哈顿距离，如果在当前轴上的单一维度距离就已经超过了 best_dist，
        // 那么远侧子树中不可能存在更近的点。
        let axis_dist = diff.abs() as usize;
        if axis_dist < *best_dist {
            if let Some(ref child) = far {
                self.search_kdtree_recursive(child, target, best_dist, best_coord);
            }
        }
    }

    /// 📏 Manhattan Distance
    fn manhattan_distance(&self, a: &Coordinate, b: &Coordinate) -> usize {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| if x > y { x - y } else { y - x })
            .sum()
    }
}
