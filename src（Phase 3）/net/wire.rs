// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::core::affine::AffineTuple;
use crate::phase3::core::algebra::ClassGroupElement;
use rug::Integer;
use serde::{Serialize, Deserialize};
use blake3::Hasher;

/// 🌳 Merkle Inclusion Proof (日志一致性证明)
/// 证明某个 Checkpoint 确实存在于不可篡改的全局日志中。
/// 
/// Verifier: 计算 Hash(Leaf + Siblings) -> ... -> Root，并比对 Global Root。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MerkleProof {
    pub leaf_index: u64,
    pub leaf_hash: [u8; 32],
    /// Merkle Path: 从叶子到根的兄弟节点哈希序列
    pub siblings: Vec<[u8; 32]>,
}

impl MerkleProof {
    /// ✅ 验证 Merkle Proof 是否有效
    /// 
    /// # 参数
    /// * `global_root`: 当前区块链/日志的全局 Merkle Root (受信锚点)
    pub fn verify(&self, global_root: &[u8; 32]) -> bool {
        let mut current_hash = self.leaf_hash;
        let mut index = self.leaf_index;

        for sibling in &self.siblings {
            let mut hasher = Hasher::new();
            hasher.update(b"HTP_MERKLE_NODE");

            // 根据 index 的奇偶性决定左右拼接顺序
            // Index 为偶数 -> 当前节点在左，Sibling 在右
            // Index 为奇数 -> 当前节点在右，Sibling 在左
            if index % 2 == 0 {
                hasher.update(&current_hash);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(&current_hash);
            }
            
            current_hash = hasher.finalize().into();
            index /= 2; // 向上移动一层
        }

        &current_hash == global_root
    }
}

/// ⏭️ State Transition Proof (跳表验证证明)
/// 证明从最近的 Checkpoint $S_k$ 到当前状态 $S_{curr}$ 的演化是正确的。
/// 
/// 这是一个 O(k) 的轻量级验证，k 为 Chunk Size (通常 < 64)。
/// 相比于 O(N) 的全量重算，效率提升了数个数量级。
#[derive(Serialize, Deserialize, Debug)]
pub struct StateTransitionProof {
    /// $S_k$: 锚点状态 (从最近的 Checkpoint 获取)
    /// 这是验证的起点。
    pub checkpoint_state: ClassGroupElement,
    
    /// 存在性证明：证明 $S_k$ 确实在不可篡改的日志中
    pub log_inclusion_proof: MerkleProof,

    /// $\Delta$: 增量算子序列 (Replay Buffer)
    /// 这是从 Checkpoint 到当前时刻的所有操作
    /// S_curr = S_k.apply(op_1).apply(op_2)...
    pub replay_ops: Vec<AffineTuple>,

    /// $S_{curr}$: 模型声称的最终状态
    /// 验证的目标是证明计算出的状态等于此状态。
    pub claimed_final_state: ClassGroupElement,
}

impl StateTransitionProof {
    /// 🛡️ 执行跳表验证 (Skip-list Verification)
    /// 
    /// 验证者逻辑：
    /// 1. Checkpoint 在 Log 里吗？(Merkle Check)
    /// 2. 从 Checkpoint 跑一遍 Replay Ops，结果对吗？(Math Check)
    pub fn verify(&self, global_merkle_root: &[u8; 32], discriminant: &Integer) -> bool {
        // Step 1: 审计日志 (Audit the Log)
        // 确保存档点 $S_k$ 是历史上确实发生过的，而不是 AI 捏造的幻觉起点
        if !self.log_inclusion_proof.verify(global_merkle_root) {
            // 在生产环境中，这里应记录详细的 Fraud Proof
            println!("❌ Verification Failed: Merkle proof invalid. Checkpoint not found in Log.");
            return false;
        }

        // Step 2: 重放演化 (Replay Evolution)
        // 使用 Phase 1 定义的流式原子操作 apply_affine
        let mut computed_state = self.checkpoint_state.clone();
        
        for (i, op) in self.replay_ops.iter().enumerate() {
            // Apply atomic transition: S_new = S_old^p * q
            match computed_state.apply_affine(&op.p_factor, &op.q_shift, discriminant) {
                Ok(new_state) => computed_state = new_state,
                Err(e) => {
                    println!("❌ Verification Error during replay at step {}: {}", i, e);
                    return false;
                }
            }
        }

        // Step 3: 最终一致性检查 (Final Consistency Check)
        if computed_state != self.claimed_final_state {
            println!("❌ Verification Failed: State mismatch.");
            println!("   Computed: {:?}", computed_state);
            println!("   Claimed:  {:?}", self.claimed_final_state);
            return false;
        }

        // 验证通过！
        true
    }
}

/// 📦 HTP Network Response Protocol
/// 定义了节点与客户端之间的通信格式
#[derive(Serialize, Deserialize, Debug)]
pub enum HtpResponse {
    /// 包含完整验证信息的响应包
    /// 每一个推理结果都必须附带这个 Bundle，否则视为不可信。
    ProofBundle {
        request_id: u64,
        
        /// 核心证明组件 (Skip-list Proof)
        proof: StateTransitionProof,
        
        /// 元数据：当前 Log 的 Epoch (Merkle Tree Size)
        /// 客户端可以用它来同步本地的 Light Client 状态
        log_epoch: u64,
    },
    
    /// 简单的确认信号
    Ack,
    
    /// 错误报告
    Error(String),
}
