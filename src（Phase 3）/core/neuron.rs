// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::core::affine::AffineTuple;
use crate::phase3::topology::tensor::HyperTensor;
use crate::phase3::net::wire::HtpResponse; 
use crate::phase3::core::algebra::ClassGroupElement;
use rug::Integer;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use rand::{Rng, thread_rng};

/// 🧠 HTPNeuron: 进化神经元 (Phase 3 Engine)
/// 实现了 "Neural Streaming" 架构，即时消耗算子，维持恒定状态大小。
pub struct HTPNeuron {
    /// [Semantic Weight]: 神经元的独特语义指纹 (大素数)
    pub p_weight: Integer,
    /// [Holographic Memory]: 用于短期上下文折叠的微型超张量
    pub memory: Arc<RwLock<HyperTensor>>,
    /// [System Param]: 判别式
    pub discriminant: Integer,
    /// [Streaming State]: 当前累积的语义状态 (Q部分)
    /// 即使处理了 100万个 Token，这个状态的大小也是恒定的 (约等于 Discriminant 位宽)
    pub semantic_root: RwLock<ClassGroupElement>,
    /// [Micro-Buffer]: 用于构建局部 Checkpoint 的微观缓冲区
    pub commitment_buffer: RwLock<Vec<AffineTuple>>,
}

impl HTPNeuron {
    pub fn new(semantic_fingerprint: Integer, dim: usize, side_len: usize, discriminant: Integer) -> Self {
        let tensor = HyperTensor::new(dim, side_len, discriminant.clone());
        HTPNeuron {
            p_weight: semantic_fingerprint,
            memory: Arc::new(RwLock::new(tensor)),
            discriminant: discriminant.clone(),
            semantic_root: RwLock::new(ClassGroupElement::identity(&discriminant)),
            commitment_buffer: RwLock::new(Vec::new()),
        }
    }

    /// ⚡ Activate: 执行流式推理
    pub fn activate(
        &self, 
        input_stream: Vec<AffineTuple>, 
        recursion_depth: usize 
    ) -> Result<(AffineTuple, HtpResponse), String> {
        
        let start_time = Instant::now();
        // 只有在局部 Buffer 满时才刷入 Tensor，这限制了 compose 的深度
        const CHUNK_SIZE: usize = 64; 

        let mut memory_guard = self.memory.write().map_err(|_| "Memory Lock poisoned")?;
        let mut s_guard = self.semantic_root.write().map_err(|_| "Semantic Root Lock poisoned")?;
        let mut buffer_guard = self.commitment_buffer.write().map_err(|_| "Buffer Lock poisoned")?;

        // Reset state for new inference pass
        *s_guard = ClassGroupElement::identity(&self.discriminant);
        buffer_guard.clear();
        
        for (t, tuple) in input_stream.iter().enumerate() {
            // (a) Blinded Evolution: 加权演化
            // S_new = S_old ^ (Tuple_P * Weight)
            let weighted_tuple = self.evolve_tuple_blinded(tuple, &self.p_weight)?;

            // (b) SpaceTime Noise: 注入时空噪声
            // 确保 S_t 与 S_{t+1} 即使输入相同也代数不同
            let time_noise = self.generate_spacetime_noise(t)?;
            let step_op = weighted_tuple.compose(&time_noise, &self.discriminant)?;

            // [Track A: Global Stream]
            // 立即应用算子，消耗 P 因子。s_guard 的大小保持不变。
            // 这里体现了 "Streaming" 的核心优势。
            *s_guard = s_guard.apply_affine(&step_op.p_factor, &step_op.q_shift, &self.discriminant)?;

            // [Track B: Local Commitment]
            // 将算子暂存，用于生成可验证的 Checkpoint
            buffer_guard.push(step_op);

            // (c) Chunking & Checkpoint
            if buffer_guard.len() >= CHUNK_SIZE || t == input_stream.len() - 1 {
                // 当 Buffer 满时，我们创建一个 Snapshot
                // 注意：Checkpoint 本身是一个 P=1 的状态点，用于索引
                let checkpoint = AffineTuple {
                    p_factor: Integer::from(1),
                    q_shift: s_guard.clone(),
                };

                let checkpoint_key = format!("chk:seq:{}", t);
                // 写入全息张量，供后续 Oracle 查询或反向解码
                memory_guard.insert(&checkpoint_key, checkpoint, t as u64)?;
                buffer_guard.clear();
            }
        }

        // 计算全息根 (Global Root)，用于一致性验证
        let _raw_tensor_root = memory_guard.calculate_global_root()?;

        // 返回最新的语义状态作为输出
        let final_output = AffineTuple {
            p_factor: Integer::from(1),
            q_shift: s_guard.clone(), 
        };

        // 构造证明包 (简化版)
        let proof_coord = memory_guard.map_id_to_coord(0); 
        let proof_path = memory_guard.get_segment_tree_path(&proof_coord, 0);
        
        let proof = HtpResponse::ProofBundle {
            request_id: 0,
            primary_path: proof_path,
            orthogonal_anchors: vec![],
            epoch: recursion_depth as u64,
        };

        Ok((final_output, proof))
    }

    /// 内部逻辑：加权与盲化
    fn evolve_tuple_blinded(&self, tuple: &AffineTuple, weight: &Integer) -> Result<AffineTuple, String> {
        let mut rng = thread_rng();
        // 简单的盲化因子，防止侧信道分析
        let blind_exp = Integer::from(rng.gen::<u64>());
        let generator = ClassGroupElement::generator(&self.discriminant);
        let r_blind = generator.pow(&blind_exp, &self.discriminant)?;
        
        // 盲化 Q -> 幂运算加权 -> 去盲化
        let q_blinded = tuple.q_shift.compose(&r_blind, &self.discriminant)?;
        let q_prime_blinded = q_blinded.pow(weight, &self.discriminant)?;
        
        // 修正项
        let r_w = r_blind.pow(weight, &self.discriminant)?;
        // 逆元: (a, -b, c)
        let r_w_inv = ClassGroupElement {
            a: r_w.a,
            b: -r_w.b, 
            c: r_w.c,
        };
        
        let new_q = q_prime_blinded.compose(&r_w_inv, &self.discriminant)?;
        let new_p = Integer::from(&tuple.p_factor * weight);

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    fn generate_spacetime_noise(&self, t: usize) -> Result<AffineTuple, String> {
        let g = ClassGroupElement::generator(&self.discriminant);
        let h_t = Integer::from(t + 1);
        let q_noise = g.pow(&h_t, &self.discriminant)?;
        Ok(AffineTuple {
            p_factor: Integer::from(1),
            q_shift: q_noise,
        })
    }
}
