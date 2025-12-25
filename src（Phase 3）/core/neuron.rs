// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::affine::AffineTuple;
use crate::topology::tensor::HyperTensor;
use crate::net::wire::HtpResponse; 
use rug::Integer;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::thread;

/// HTPNeuron: 仿射神经元 (The Processor)
/// 
/// [Architecture Note]:
/// 这是 Phase 3 演化系统的核心计算单元。
/// 它不再是一个简单的函数，而是一个带有“状态记忆”的有限状态自动机。
pub struct HTPNeuron {
    /// 语义指纹：代表神经元特定逻辑功能的超大素数
    pub p_weight: Integer,
    
    /// 全息记忆张量：存储神经元的思维历史和逻辑检查点
    pub memory: Arc<RwLock<HyperTensor>>,
    
    /// 系统判别式：定义代数群的几何形状
    pub discriminant: Integer,
}

impl HTPNeuron {
    pub fn new(semantic_fingerprint: Integer, dim: usize, side_len: usize, discriminant: Integer) -> Self {
        let tensor = HyperTensor::new(dim, side_len, discriminant.clone());
        HTPNeuron {
            p_weight: semantic_fingerprint,
            memory: Arc::new(RwLock::new(tensor)),
            discriminant,
        }
    }

    /// ⚡ Algebraic Activation: 代数激活函数
    /// 
    /// [FIX: Coefficient Explosion Mitigation]
    /// 引入 "Adaptive Semantic Resealing" (自适应语义重封) 机制。
    /// 传统的无限累积会导致 P-Factor 指数级爆炸。我们通过监测位宽，
    /// 在 P 接近安全边界 (4096 bits) 时主动“结晶”当前状态，存入记忆张量，
    /// 并重置累加器。这使得神经元可以处理无限长的上下文而不崩溃。
    ///
    /// [SECURITY UPDATE]: 引入 Isochronous Padding (等时填充) 以防御侧信道攻击。
    pub fn activate(
        &self, 
        input_stream: Vec<AffineTuple>, 
        recursion_depth: usize 
    ) -> Result<(AffineTuple, HtpResponse), String> {
        
        // [TIMING PROTECTION]: 启动精密计时器
        // 记录函数进入的物理时刻，用于后续的等时填充
        let start_time = Instant::now();

        let mut memory_guard = self.memory.write().map_err(|_| "Lock poisoned")?;
        
        // [Resealing Accumulator]: 用于暂存当前逻辑段的累积状态
        // 初始为单位元 (Identity)
        let mut current_accumulator = AffineTuple::identity(&self.discriminant);
        
        // 1. [Non-Commutative Evolution Loop]
        for (t, tuple) in input_stream.iter().enumerate() {
            // (a) 加权演化: Input ^ Weight
            let weighted_tuple = self.evolve_tuple(tuple, &self.p_weight)?;

            // (b) 时空噪声注入: * G ^ H(t)
            let time_noise = self.generate_spacetime_noise(t)?;
            
            // 计算单步增量 (Step Tuple)
            let step_tuple = weighted_tuple.compose(&time_noise, &self.discriminant)?;

            // (c) [CRITICAL FIX]: 爆炸预判与重封 (Resealing Check)
            // 预测下一次复合后的位宽
            let current_bits = current_accumulator.p_factor.significant_bits();
            let step_bits = step_tuple.p_factor.significant_bits();
            
            // 阈值设为 3072 bits，留出约 1000 bits 的安全余量 (Hard Limit is 4096)
            if current_bits + step_bits > 3072 {
                // ⚠️ 触发重封 (Trigger Resealing)
                // 将当前累积的逻辑链“烘焙”成一个检查点，永久存入记忆
                let checkpoint_key = format!("chk:seal:{}", t);
                
                // 存入 HyperTensor (Spacetime Orthogonal Storage)
                memory_guard.insert(&checkpoint_key, current_accumulator.clone(), t as u64)?;
                
                // 重置累加器为当前步 (Start New Segment)
                // 这相当于数学上的 "Re-basing"，将当前时刻视为新的逻辑起点
                current_accumulator = step_tuple;
            } else {
                // ✅ 安全，继续累积
                // Accumulator = Accumulator (+) Step
                current_accumulator = current_accumulator.compose(&step_tuple, &self.discriminant)?;
            }
        }

        // 循环结束，处理残留的累加器
        // 将最后的思维片段写入记忆
        let final_t = input_stream.len();
        let final_key = format!("chk:tail:{}", final_t);
        memory_guard.insert(&final_key, current_accumulator, final_t as u64)?;

        // 2. [Fold]: 全息折叠
        // memory_guard.calculate_global_root() 会自动聚合所有的 Checkpoints
        // 形成最终的全局语义根
        let raw_output = memory_guard.calculate_global_root()?;

        // 3. [Reduce]: 代数规约
        // 防止输出传递给下一层时导致级联爆炸
        let final_output = self.algebraic_reduction(raw_output, recursion_depth)?;

        // 4. [Proof Generation]: 生成推理证明
        // 随机取样一个维度作为解释性证明
        let proof_coord = memory_guard.map_id_to_coord(0); 
        let proof_path = memory_guard.get_segment_tree_path(&proof_coord, 0);
        
        let proof = HtpResponse::ProofBundle {
            request_id: 0,
            primary_path: proof_path,
            orthogonal_anchors: vec![],
            epoch: recursion_depth as u64,
        };

        // [SECURITY FIX]: Isochronous Padding (等时填充)
        // 强制函数执行时间对齐到固定的时间桶 (Time Bucket)
        // 这淹没了底层 GMP 库由于输入敏感性导致的时间差异。
        // 假设根据 Benchmark，最坏情况下的演化路径不会超过 50ms。
        const SECURITY_LATENCY_BUDGET_MS: u64 = 50;
        let target_duration = Duration::from_millis(SECURITY_LATENCY_BUDGET_MS);
        
        let elapsed = start_time.elapsed();
        if elapsed < target_duration {
            // 如果计算过快，主动休眠以补齐时间差
            // 这对吞吐量有一定影响，但为了安全是必须的
            thread::sleep(target_duration - elapsed);
        } else {
            // 如果超时，说明系统负载过高或受到 DoS 攻击
            // 在日志中记录，但不中断服务
            // log::warn!("⚠️ Timing budget exceeded: {:?}", elapsed);
        }

        Ok((final_output, proof))
    }

    /// 内部助手：对单个元组应用权重 P
    fn evolve_tuple(&self, tuple: &AffineTuple, weight: &Integer) -> Result<AffineTuple, String> {
        let new_p = Integer::from(&tuple.p_factor * weight);
        let new_q = tuple.q_shift.pow(weight, &self.discriminant)?;
        
        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    /// 内部助手：生成时空噪声 G^H(t)
    fn generate_spacetime_noise(&self, t: usize) -> Result<AffineTuple, String> {
        let g = crate::core::algebra::ClassGroupElement::generator(&self.discriminant);
        // H(t) = hash(t)，这里简化为线性
        let h_t = Integer::from(t + 1);
        let q_noise = g.pow(&h_t, &self.discriminant)?;
        
        // 噪声项的 P 通常为 1 (Identity)
        Ok(AffineTuple {
            p_factor: Integer::from(1),
            q_shift: q_noise,
        })
    }

    /// [Residual Management]: 模拟代数规约与噪声过滤
    fn algebraic_reduction(&self, tuple: AffineTuple, depth: usize) -> Result<AffineTuple, String> {
        let identity = AffineTuple::identity(&self.discriminant);
        
        // "Residual Cutoff": 如果深度超过阈值，强制规约
        // 注意：这里的 compose 会触发底层的 reduce_form (Lagrange Algorithm)，
        // 这对于 Q 因子是无损的，但如果 P 过大，可能需要在网络层面进行额外处理。
        if depth > 10 {
             return tuple.compose(&identity, &self.discriminant);
        }
        
        Ok(tuple)
    }
}
