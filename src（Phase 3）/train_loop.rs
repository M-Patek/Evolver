// src/phase3/train_loop.rs
// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::structure::HTPModel;
use crate::phase3::decoder::InverseDecoder;
use crate::core::primes::hash_to_prime;
use std::sync::{Arc, RwLock};
use rand::Rng;

/// 🧬 EvolutionaryTrainer: 进化训练器
pub struct EvolutionaryTrainer {
    /// 模型本身被 RwLock 保护，以便我们可以修改其结构或参数
    pub model: Arc<RwLock<HTPModel>>,
    pub decoder: InverseDecoder,
    pub learning_rate: f64, // 基础突变概率
}

impl EvolutionaryTrainer {
    pub fn new(model: Arc<RwLock<HTPModel>>, vocab_size: u32) -> Self {
        EvolutionaryTrainer {
            model,
            decoder: InverseDecoder::new(vocab_size),
            learning_rate: 0.05, // 5% 的概率发生突变
        }
    }

    /// 🏋️ Train Step: 单步进化循环
    /// 引入了 "Zero-Tolerance Drift" 机制
    pub fn train_step(&mut self, input_ids: &[u32], target_id: u32) -> Result<f32, String> {
        // [Step 1]: Forward Pass (推理)
        let prediction_root = {
            let model_guard = self.model.read().map_err(|_| "Model Lock Poisoned")?;
            model_guard.forward(input_ids)?
        };

        // [Step 2]: Decode & Drift Check (验证与探针)
        // 这里的 unwrap_or 只是为了处理完全迷航的情况
        let decode_result = self.decoder.decode(&prediction_root)
            .unwrap_or(crate::phase3::decoder::DecodeResult { token_id: u32::MAX, drift: usize::MAX });

        let is_target_hit = decode_result.token_id == target_id;
        let mut loss = 0.0;

        // [Step 3]: Evolution Strategy (进化策略)
        
        // Case A: 完全错误 -> 死刑 (Punish Mutation)
        if !is_target_hit {
            loss = 1.0;
            self.punish_path_mutation();
        } 
        // Case B: 命中但存在漂移 -> 精确性压力 (Precision Pressure)
        else if decode_result.drift > 0 {
            // 虽然对了，但是是有偏差的。给予一个较小的 Loss 警示。
            loss = 0.1 * (decode_result.drift as f32);
            
            // 计算“精确性风险”：漂移越大，触发微扰突变的概率越高
            // 例如：漂移 10 个单位，就有 10% * 0.5 = 5% 的概率被重置
            // 这迫使网络向“漂移为 0”的完美状态收敛
            let drift_risk = (decode_result.drift as f64) * 0.05; 
            
            let mut rng = rand::thread_rng();
            if rng.gen_bool(drift_risk.min(0.5)) { // 风险封顶 50%
                self.apply_micro_mutation();
            } else {
                // 如果侥幸逃脱突变，我们也可以视为一种弱奖励（保留现状）
                // 但长远来看，漂移是不稳定的
            }
        }
        // Case C: 完美命中 (Zero Drift) -> 奖励 (Reward)
        else {
            loss = 0.0;
            self.reward_path();
        }

        Ok(loss)
    }

    fn reward_path(&self) {
        // 正确且精准的路径被保留。
        // println!("✨ Perfect Logic Path Validated (Zero Drift).");
    }

    /// ☣️ Hard Mutation: 彻底重置
    /// 用于处理严重的逻辑错误 (Hallucination)
    fn punish_path_mutation(&mut self) {
        self.mutate_network(true);
    }

    /// 🔬 Micro Mutation: 微扰突变
    /// 用于消除漂移 (Drift)。
    /// 在逻辑上，这可能尝试在当前语义指纹附近寻找更优解，
    /// 或者仅仅是以较低的烈度触发重置，试图 "Shake" 网络进入更好的局部最优。
    fn apply_micro_mutation(&mut self) {
        // println!("⚠️ Drift Detected. Applying Micro-Mutation...");
        // 这里的 false 标志位可以用于未来控制突变的幅度
        // 目前为了保证代数性质的完整性，我们依然使用重哈希，但可以在 log 中区分
        self.mutate_network(false); 
    }

    /// 通用突变逻辑
    fn mutate_network(&mut self, is_hard_reset: bool) {
        let mut rng = rand::thread_rng();
        let mut model_guard = self.model.write().expect("Model Lock Poisoned during mutation");

        for layer in &mut model_guard.layers {
            for neuron_lock in &layer.neurons {
                // 如果是 Hard Reset，使用标准学习率
                // 如果是 Micro Mutation，我们可能希望更聚焦，或者通过外部概率控制（外部已控制）
                if rng.gen_bool(self.learning_rate) {
                    
                    let mut neuron_mut = neuron_lock.write().expect("Neuron Lock Poisoned");

                    // 构造新的种子
                    // Micro Mutation 可以尝试混入之前的权重特征，试图保留部分语义 (TODO)
                    // 目前实现为随机搜索 (Stochastic Search)
                    let mutation_type = if is_hard_reset { "HARD" } else { "MICRO" };
                    let new_seed = format!("{}_mut_{}_{}", 
                        mutation_type,
                        rng.gen::<u64>(), 
                        neuron_mut.discriminant
                    );

                    match hash_to_prime(&new_seed, 128) {
                        Ok(new_prime) => {
                            neuron_mut.p_weight = new_prime;
                            if let Ok(mut memory_guard) = neuron_mut.memory.write() {
                                memory_guard.data.clear();
                                memory_guard.cached_root = None;
                            }
                        },
                        Err(_) => continue,
                    }
                }
            }
        }
    }
}
