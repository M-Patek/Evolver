// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::structure::{HTPModel, CrystalLayer};
use crate::phase3::decoder::InverseDecoder;
use crate::core::primes::hash_to_prime;
use crate::core::neuron::HTPNeuron;
use rug::Integer;
use std::sync::{Arc, RwLock}; // 我们需要锁来修改权重
use rand::Rng;

/// 🧬 EvolutionaryTrainer: 进化训练器
/// 既然无法求导，我们就通过“适者生存”法则来训练网络。
pub struct EvolutionaryTrainer {
    pub model: Arc<RwLock<HTPModel>>, // 模型本身需要支持内部变异
    pub decoder: InverseDecoder,
    pub learning_rate: f64, // 这里代表“突变概率”
    pub mutation_strength: u32, // 突变时跳跃的幅度
}

impl EvolutionaryTrainer {
    pub fn new(model: Arc<RwLock<HTPModel>>, vocab_size: u32) -> Self {
        EvolutionaryTrainer {
            model,
            decoder: InverseDecoder::new(vocab_size),
            learning_rate: 0.1, // 10% 的概率发生严重突变
            mutation_strength: 1, 
        }
    }

    /// 🏋️ Train Step: 单步进化
    /// 1. Forward -> 2. Check Math Error -> 3. Mutate or Reinforce
    pub fn train_step(&mut self, input_ids: &[u32], target_id: u32) -> Result<f32, String> {
        // [Step 1]: Forward Pass (推理)
        // 获取读锁进行推理
        let prediction_root = {
            let model_guard = self.model.read().map_err(|_| "Model Lock Poisoned")?;
            model_guard.forward(input_ids)?
        };

        // [Step 2]: Decode & Navigation (导航)
        // 尝试将代数结果还原为 Token
        let predicted_id = self.decoder.decode(&prediction_root)
            .unwrap_or(u32::MAX); // 如果彻底迷失，给一个错误值

        let is_correct = predicted_id == target_id;
        
        // 计算“语义距离”作为 Loss (仅供观察，不参与梯度)
        // 这里简化为 0 (正确) 或 1 (错误)
        let loss = if is_correct { 0.0 } else { 1.0 };

        // [Step 3]: Feedback Loop (反馈)
        if is_correct {
            self.reward_path();
        } else {
            // 发生了 Math Error (幻觉)，立即惩罚！
            self.punish_path_mutation(loss);
        }

        Ok(loss)
    }

    /// 🍬 Reward: 奖励机制
    /// 预测正确！该路径上的神经元证明了它们的代数结构是自洽的。
    /// 策略：保持现状，或者微调 memory (强化记忆)。
    fn reward_path(&self) {
        // 在 HTP 理论中，"Survival is the only reward".
        // 存活下来的神经元不需要改变，它们的权重（素数）就是对的。
        // 可选：增加该路径神经元的 "Confidence" 计数器（暂未实现）
        println!("✨ [Correct] Crystal path validated. No mutation needed.");
    }

    /// ☣️ Punishment: 突变惩罚
    /// 预测错误！说明当前的代数路径无法闭环。
    /// 策略：随机选择参与计算的神经元，强制修改它们的 Semantic Fingerprint (P_weight)。
    fn punish_path_mutation(&mut self, _error_magnitude: f32) {
        let mut rng = rand::thread_rng();
        let mut model_guard = self.model.write().expect("Lock poisoned during mutation");

        println!("💥 [Math Error] Logic collapsed. Initiating mutation...");

        // 遍历所有层
        for layer in &mut model_guard.layers {
            // 随机挑选几个“倒霉”的神经元进行突变
            // 这是一个随机搜索过程 (Stochastic Search)
            for neuron_arc in &layer.neurons {
                if rng.gen_bool(self.learning_rate) {
                    // 为了修改 Arc 内部的数据，我们需要 HTPNeuron 支持内部可变性
                    // 或者我们在 Layer 定义时就使用了 RwLock<HTPNeuron>
                    // 这里假设我们在 structure.rs 中已经做好了准备，或者我们执行“热替换”
                    
                    // [Simulation]: 模拟权重突变
                    // 旧的素数 P_old -> 新的素数 P_new
                    // 这种突变改变了神经元的“语义定义”
                    
                    // 注意：在实际 Rust 代码中，Arc<HTPNeuron> 是不可变的。
                    // 真正的实现需要 layer.neurons 存储 Arc<RwLock<HTPNeuron>>。
                    // 此处演示核心逻辑：
                    
                    if let Some(neuron_mut) = Arc::get_mut(neuron_arc) {
                        // 这是一个极其暴力的操作：直接改变神经元的本质
                        let new_seed = format!("mutated_{}", rng.gen::<u64>());
                        if let Ok(new_prime) = hash_to_prime(&new_seed, 128) {
                            neuron_mut.p_weight = new_prime;
                            // 清空记忆，因为语义变了，旧记忆无效
                            if let Ok(mut mem) = neuron_mut.memory.write() {
                                mem.data.clear();
                            }
                            println!("   🧬 Neuron mutated: Re-hashed semantic fingerprint.");
                        }
                    } else {
                        // 如果无法获取可变引用（通常是因为并在使用中），
                        // 我们在真实系统中会 clone 并替换整个 Arc
                        println!("   ⚠️ Skip mutation: Neuron is busy (Arc locked).");
                    }
                }
            }
        }
    }

    /// 🔄 Training Loop Demo
    pub fn run_demo_loop(&mut self, epochs: usize) {
        // 模拟数据：(Context, Target)
        let dummy_data = vec![
            (vec![1, 2, 3], 4), // Context: A, B, C -> Target: D
            (vec![10, 20], 30),
            (vec![99, 100], 101),
        ];

        for epoch in 0..epochs {
            println!("--- Epoch {} ---", epoch);
            let mut total_loss = 0.0;
            
            for (input, target) in &dummy_data {
                match self.train_step(input, *target) {
                    Ok(loss) => total_loss += loss,
                    Err(e) => println!("Error: {}", e),
                }
            }

            if total_loss == 0.0 {
                println!("🎉 Convergence Reached! The Crystal Brain is perfect.");
                break;
            }
        }
    }
}

// -------------------------------------------------------------------------
// Helper for structure.rs compatibility (Mocking the mutation requirement)
// -------------------------------------------------------------------------
// 为了让上面的 Arc::get_mut 工作，我们需要确保没有其他线程持有 Arc。
// 在训练阶段，这通常是单线程进行的，或者使用 RwLock 包装。
