// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use blake3::Hasher;
use serde::{Serialize, Deserialize};

/// 🌳 Incremental Merkle Tree (增量 Merkle 树)
/// 专为 Append-only Log 设计，支持动态添加叶子节点并快速计算 Root。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IncrementalMerkleTree {
    /// 每一层的尾部节点 (用于快速合并)
    /// peaks[i] 存储的是高度为 i 的最右侧子树的 Root
    pub peaks: Vec<Option<[u8; 32]>>,
    /// 当前叶子总数
    pub leaf_count: u64,
}

impl IncrementalMerkleTree {
    pub fn new() -> Self {
        IncrementalMerkleTree {
            peaks: Vec::new(),
            leaf_count: 0,
        }
    }

    /// 🌱 Append: 添加一个新的叶子 Hash
    pub fn append(&mut self, leaf_hash: [u8; 32]) {
        let mut current_hash = leaf_hash;
        let mut height = 0;

        // 增量合并逻辑：
        // 如果当前高度已经有 Peak，说明该层已满，需要合并并上升到下一层
        // 如果当前高度没有 Peak，直接放入
        loop {
            if height >= self.peaks.len() {
                self.peaks.push(None);
            }

            match self.peaks[height] {
                Some(left_sibling) => {
                    // Merge (Left + Right) -> Parent
                    current_hash = self.hash_node(&left_sibling, &current_hash);
                    self.peaks[height] = None; // 该层清空，向上进位
                    height += 1;
                }
                None => {
                    // 找到空位，在此停留
                    self.peaks[height] = Some(current_hash);
                    break;
                }
            }
        }
        self.leaf_count += 1;
    }

    /// 👑 Calculate Root: 计算当前的 Merkle Root
    pub fn root(&self) -> [u8; 32] {
        if self.leaf_count == 0 {
            return [0u8; 32];
        }

        let mut root_hash = [0u8; 32];
        let mut first = true;

        // 从低向高合并所有的 Peaks
        for peak in self.peaks.iter() {
            if let Some(h) = peak {
                if first {
                    root_hash = *h;
                    first = false;
                } else {
                    // 注意：由于 Peaks 是从右向左积累的结构，这里的合并顺序需要小心
                    // 但对于 Accumulator 来说，我们只要保证确定性即可
                    root_hash = self.hash_node(&root_hash, h); 
                }
            }
        }
        root_hash
    }

    fn hash_node(&self, left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(b"HTP_MERKLE_NODE");
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
}
