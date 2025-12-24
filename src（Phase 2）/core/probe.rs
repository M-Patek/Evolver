// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::affine::AffineTuple;
use crate::core::neuron::HTPNeuron;
use crate::core::oracle::HTPOracle;
use crate::core::primes::hash_to_prime;
use crate::core::algebra::ClassGroupElement;
use rug::Integer;
use std::sync::Arc;

/// 🕵️ HTPProbe: 语义宪兵队
/// 它的职责不是生成，而是“监察” Transformer 的 Hidden States。
/// 集成了 Oracle 用于快速验证。
pub struct HTPProbe {
    /// 绑定的神经元（负责具体的代数演化计算）
    neuron: Arc<HTPNeuron>,
    
    /// [Oracle Integration]: 代数预言机，用于 O(1) 验证
    oracle: HTPOracle,

    /// 阈值灵敏度：决定多少概率的 Attention 值得被转化为“硬逻辑”
    /// 范围 [0.0, 1.0]，默认 0.1
    attention_threshold: f32,
}

impl HTPProbe {
    pub fn new(neuron: Arc<HTPNeuron>, threshold: f32) -> Self {
        let oracle = HTPOracle::new(neuron.clone());
        HTPProbe {
            neuron,
            oracle,
            attention_threshold: threshold,
        }
    }

    /// 🔄 1. Attention-to-Prime Converter
    /// 将 Transformer 的注意力分布转化为代数输入流
    pub fn quantize_attention(
        &self, 
        token_ids: &[u32], 
        attention_weights: &[f32]
    ) -> Result<Vec<AffineTuple>, String> {
        if token_ids.len() != attention_weights.len() {
            return Err("Dimension mismatch between tokens and weights".into());
        }

        let mut algebraic_stream = Vec::new();

        for (i, &weight) in attention_weights.iter().enumerate() {
            // [Filter]: 只有权重超过阈值的 Token 才有资格参与逻辑演化
            // 这是一个 "Soft-to-Hard" 的关键转换点
            if weight > self.attention_threshold {
                let token_id_str = format!("tok_{}", token_ids[i]);
                
                // [Mapping]: Token ID -> Prime (P)
                let p = hash_to_prime(&token_id_str, 64).map_err(|e| e.to_string())?;
                
                // [Mapping]: Weight -> Power (Optional)
                // 我们可以让权重影响演化的深度，或者简单地作为开关。
                // 这里为了简化，只要通过阈值，就视为有效算子。
                
                // 构造对应的 AffineTuple，假设 Q 为 Generator (代表标准语义方向)
                let q = ClassGroupElement::generator(&self.neuron.discriminant);
                
                algebraic_stream.push(AffineTuple {
                    p_factor: p,
                    q_shift: q,
                });
            }
        }
        
        Ok(algebraic_stream)
    }

    /// 🛡️ 2. The Logic Validator (Forward Pass)
    /// 验证：给定当前上下文，Transformer 预测的 'next_token' 是否合法？
    /// [Optimized]: 使用 Oracle 进行 O(1) 查找，取代了 Phase 2 的暴力计算。
    pub fn verify_inference(
        &self,
        context_stream: Vec<AffineTuple>,
        next_token_id: u32
    ) -> Result<f32, String> {
        // Step A: 运行 HTP 神经元的演化，激活内部记忆张量
        // 这会更新 Neuron 内部的 Tensor 状态
        let (_expected_state, _proof) = self.neuron.activate(context_stream, 1)?;
        
        // Step B: 调用 Oracle 提取当前上下文的合法候选集
        // 这是 O(Active_Memory) 的操作，远快于遍历词表
        let candidates = self.oracle.suggest_candidates()?;
        
        // Step C: 将 Transformer 预测的 Token 转化为素数
        let token_str = format!("tok_{}", next_token_id);
        let candidate_p = hash_to_prime(&token_str, 64).map_err(|e| e.to_string())?;
        
        // Step D: O(1) 集合查询
        if candidates.contains(&candidate_p) {
            // 命中！绝对合法的代数后继
            Ok(1.0)
        } else {
            // 未命中。
            // 可能是幻觉，也可能是该概念从未在上下文中出现过（Out-of-Distribution）。
            // 我们给予严厉的惩罚。
            Ok(0.01)
        }
    }

    /// 🚫 3. The Veto Mechanism (阻断机制)
    /// 修改 Logits，根据逻辑置信度进行惩罚
    pub fn apply_veto(
        &self,
        original_logits: &mut [f32],
        token_ids: &[u32],
        logic_scores: &[f32]
    ) {
        // alpha: 逻辑惩罚系数。越大则 HTP 对幻觉的容忍度越低。
        let alpha = 5.0; 

        for (i, &score) in logic_scores.iter().enumerate() {
            if score < 0.5 {
                // 如果逻辑置信度低，大幅降低 Logit
                // Logit = Logit - alpha * (1 - score)
                original_logits[i] -= alpha * (1.0 - score);
            }
            // 如果逻辑置信度高，保持不变（或者微弱奖励）
        }
    }
}
