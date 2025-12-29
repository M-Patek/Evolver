// New Evolver Phase 2: Bias Channel Controller
// 这是一个 "Sidecar" 控制器，通过低维偏置向量 (Bias Vector) 实时干预生成器的 Logits。
// 它利用 VAPO 算法在约束空间 (STP/p-adic) 中搜索最优的控制量 b。

use crate::dsl::schema::ProofAction;
use crate::dsl::stp_bridge::STPContext;
use rand::Rng;

// 假设词表大小或动作空间大小
const ACTION_SPACE_SIZE: usize = 1024; 
// 控制向量的维度 (低维控制，例如 16 或 32)
const BIAS_DIM: usize = 16;

/// 偏置向量 (Bias Vector)
/// 使用整数向量模拟 p-进整数的位数操作，符合 VAPO 的离散优化特性。
#[derive(Debug, Clone)]
pub struct BiasVector {
    pub data: Vec<i32>, // 每一位对应一个控制维度的值
}

impl BiasVector {
    pub fn new() -> Self {
        // 初始化为零偏置
        BiasVector {
            data: vec![0; BIAS_DIM],
        }
    }

    /// 将低维 Bias 投影到高维 Logits 空间 (模拟 W_proj * b)
    /// 这里使用简单的哈希映射模拟投影，实际项目中应为训练好的线性层
    pub fn project_to_logits(&self) -> Vec<f64> {
        let mut logits_bias = vec![0.0; ACTION_SPACE_SIZE];
        for (i, &val) in self.data.iter().enumerate() {
            // 简单的散列投影模拟：每一维 bias 影响一部分 logits
            let start_idx = (i * ACTION_SPACE_SIZE / BIAS_DIM);
            let end_idx = ((i + 1) * ACTION_SPACE_SIZE / BIAS_DIM).min(ACTION_SPACE_SIZE);
            
            for k in start_idx..end_idx {
                // 模拟某种线性关系
                logits_bias[k] += (val as f64) * 0.1; 
            }
        }
        logits_bias
    }
}

/// VAPO 优化器配置
pub struct VapoConfig {
    pub max_iterations: usize, // 最大搜索步数 (实时性要求高，不能太大)
    pub initial_temperature: f64,
    pub valuation_decay: f64, // 估值衰减系数
}

impl Default for VapoConfig {
    fn default() -> Self {
        VapoConfig {
            max_iterations: 50,
            initial_temperature: 1.0,
            valuation_decay: 0.9,
        }
    }
}

/// Bias Channel 控制器
pub struct BiasController {
    current_bias: BiasVector,
    config: VapoConfig,
}

impl BiasController {
    pub fn new(config: Option<VapoConfig>) -> Self {
        BiasController {
            current_bias: BiasVector::new(),
            config: config.unwrap_or_default(),
        }
    }

    /// VAPO 核心循环：搜索最优 Bias 以最小化 STP 能量
    /// 
    /// # 参数
    /// - `base_logits`: 生成器原始输出的 Logits
    /// - `stp_ctx`: 代数状态上下文，用于计算能量
    /// - `decoder_simulation`: 一个闭包，模拟 "Logits -> ProofAction" 的解码过程
    /// 
    /// # 返回
    /// - `BiasVector`: 修正后的偏置向量
    /// - `ProofAction`: 修正后的动作
    pub fn optimize<F>(
        &mut self,
        base_logits: &[f64],
        stp_ctx: &mut STPContext,
        decode_fn: F,
    ) -> (BiasVector, ProofAction) 
    where
        F: Fn(&[f64]) -> ProofAction, // 模拟解码器：Logits -> Action
    {
        let mut best_bias = self.current_bias.clone();
        let mut best_action = decode_fn(&Self::apply_bias(base_logits, &best_bias));
        let mut min_energy = stp_ctx.calculate_energy(&best_action);

        // 如果初始能量已经是 0 (完美逻辑)，直接返回
        if min_energy <= 1e-6 {
            return (best_bias, best_action);
        }

        let mut rng = rand::thread_rng();
        let mut temperature = self.config.initial_temperature;

        // VAPO 搜索循环
        for _iter in 0..self.config.max_iterations {
            // 1. 生成扰动 (Perturbation)
            // 根据当前能量决定扰动幅度 (Valuation-Adaptive)
            // 能量越大，我们越倾向于修改 Bias 的 "高位" (大幅度变动)
            // 能量越小，我们越倾向于修改 Bias 的 "低位" (微调)
            let mut candidate_bias = best_bias.clone();
            
            // 随机选择一个维度进行修改
            let dim_idx = rng.gen_range(0..BIAS_DIM);
            
            // VAPO 逻辑：扰动强度与能量成正比
            let perturbation_strength = if min_energy > 1.0 {
                // 严重逻辑错误 -> 大幅跳变 (模拟 p-adic 低估值位变动)
                rng.gen_range(-5..=5)
            } else {
                // 轻微偏差 -> 微调 (模拟 p-adic 高估值位变动)
                rng.gen_range(-1..=1)
            };
            
            candidate_bias.data[dim_idx] += perturbation_strength;

            // 2. 应用 Bias 并解码
            let modified_logits = Self::apply_bias(base_logits, &candidate_bias);
            let candidate_action = decode_fn(&modified_logits);

            // 3. 计算新能量
            let new_energy = stp_ctx.calculate_energy(&candidate_action);

            // 4. 接受准则 (Metropolis-Hastings 风格，防止陷入局部最优)
            let delta_e = new_energy - min_energy;
            if delta_e < 0.0 || rng.gen::<f64>() < (-delta_e / temperature).exp() {
                best_bias = candidate_bias;
                min_energy = new_energy;
                best_action = candidate_action;
                
                // 找到完美解，立即提前退出
                if min_energy <= 1e-6 {
                    break;
                }
            }

            // 降温
            temperature *= self.config.valuation_decay;
        }

        // 更新控制器的内部状态，保持连续性
        self.current_bias = best_bias.clone();
        
        (best_bias, best_action)
    }

    /// 将 Bias 叠加到 Base Logits 上
    fn apply_bias(base: &[f64], bias: &BiasVector) -> Vec<f64> {
        let bias_proj = bias.project_to_logits();
        base.iter().zip(bias_proj.iter())
            .map(|(b, p)| b + p)
            .collect()
    }
}

// -------------------------------------------------------------------------
// Mock Test
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::schema::ProofAction;
    use crate::dsl::stp_bridge::STPContext;

    #[test]
    fn test_vapo_correction() {
        // 1. 初始化环境
        let mut stp_ctx = STPContext::new();
        // 预定义 n 为 Odd
        stp_ctx.calculate_energy(&ProofAction::Define { 
            symbol: "n".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        });

        let mut controller = BiasController::new(None);
        
        // 2. 模拟 Base Logits
        // 假设生成器倾向于生成一个错误的 Action (例如 Apply ModAdd 得到 Even，但其实应该是 Odd+Odd=Even... 等等)
        // 这里简化：假设 Action Space 0 是 "错误的 Apply", Action Space 1 是 "正确的 Apply"
        let mut base_logits = vec![0.0; ACTION_SPACE_SIZE];
        base_logits[0] = 10.0; // 错误动作概率极高
        base_logits[1] = 5.0;  // 正确动作概率低

        // 3. 定义 Mock 解码器
        let decode_fn = |logits: &[f64]| -> ProofAction {
            // 简单的 Argmax
            let max_idx = logits.iter().enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();

            if max_idx == 0 {
                // 错误动作: 定义 sum 为 Odd (逻辑违规: Odd + Odd = Even)
                ProofAction::Define { 
                    symbol: "sum".to_string(), 
                    hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
                }
            } else {
                // 正确动作 (假设通过 Bias 提升 index 1 的概率)
                ProofAction::Define { 
                    symbol: "sum".to_string(), 
                    hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Even".to_string()] 
                }
            }
        };

        // 为了测试，我们需要让 "sum" 的预期状态在 STP 中是 Even
        // 手动注入预期: Odd(n) + Odd(n) -> Even
        // 但这里我们只测试 Define 动作的能量。
        // STP Context 里预设 path "Odd" -> delta_2^2, "Even" -> delta_2^1
        // 我们假设 sum 必须是 Even (通过某种外部定理约束，或者这里简单测试 Define 的一致性)
        // *为了让测试跑通，我们手动在该测试用例中 hack 一下 energy 逻辑，或者依赖 stp_bridge 的 Define 逻辑*
        // stp_bridge 的 Define 逻辑是：如果 sum 已存在，检查漂移。
        // 让我们先 Apply 一个 ModAdd 让 sum 变成 Even
        stp_ctx.calculate_energy(&ProofAction::Apply { 
            theorem_id: "ModAdd".to_string(), 
            inputs: vec!["n".to_string(), "n".to_string()], // Odd + Odd
            output_symbol: "sum".to_string() 
        }); 
        // 此时 sum 在 ctx 中是 Even。
        
        // 4. 运行 VAPO 优化
        println!("Starting optimization...");
        let (final_bias, final_action) = controller.optimize(&base_logits, &mut stp_ctx, decode_fn);

        // 5. 验证结果
        // 初始是 index 0 (Define sum Odd) -> Energy 高 (因为 ctx 中 sum 是 Even)
        // 期望 VAPO 找到 bias 使得 index 1 (Define sum Even) 概率最大 -> Energy 0
        
        match final_action {
            ProofAction::Define { hierarchy_path, .. } => {
                assert_eq!(hierarchy_path.last().unwrap(), "Even", "VAPO failed to correct the logic!");
                println!("VAPO successfully corrected the action to Even!");
            },
            _ => panic!("Unexpected action type"),
        }
    }
}
