// src/control/bias_channel.rs
// 这是一个 "Sidecar" 控制器，通过低维偏置向量 (Bias Vector) 实时干预生成器的 Logits。
// 它利用 VAPO 算法在约束空间 (STP/p-adic) 中搜索最优的控制量 b。

use crate::dsl::schema::ProofAction;
use crate::dsl::stp_bridge::STPContext;
use rand::Rng;
use rand::SeedableRng; // Needed for BiasProjector reproducibility
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap; // For Caching
use std::hash::{Hash, Hasher};
use std::f64::consts::PI;

// 假设词表大小或动作空间大小
const ACTION_SPACE_SIZE: usize = 1024;
// 控制向量的维度 (低维控制，例如 16 或 32)
const BIAS_DIM: usize = 16;
// 环面模数 L (Bias Ring Size)
// 这定义了坐标环的大小 Z/LZ。这也决定了 VAPO 的搜索边界。
const BIAS_RING_SIZE: i32 = 256;

// =========================================================================
// Bias Projector (Fixed Random Projection)
// =========================================================================

/// 偏置投影器
/// 负责将低维的 Bias 向量映射到高维的 Action Logits 空间。
/// 为了保证系统的确定性和可复现性，我们使用固定的随机种子初始化权重矩阵。
#[derive(Clone)]
pub struct BiasProjector {
    w: Vec<Vec<f64>>, // 权重矩阵 [ACTION_SPACE_SIZE][2*BIAS_DIM]
    scale: f64,       // 缩放因子
}

impl BiasProjector {
    pub fn new() -> Self {
        // 使用固定种子 (0xC0FFEE) 确保每次启动行为一致
        // 这样 Bias Vector 的语义在不同运行之间是稳定的
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xC0FFEE);
        let mut w = vec![vec![0.0; 2 * BIAS_DIM]; ACTION_SPACE_SIZE];

        for k in 0..ACTION_SPACE_SIZE {
            for j in 0..2 * BIAS_DIM {
                w[k][j] = rng.gen_range(-1.0..1.0);
            }
        }

        // Scale 稍微放大一点，让较小的 Bias 也能产生足够的 Logits 变化
        Self { w, scale: 2.0 }
    }
}

// =========================================================================
// Bias Vector
// =========================================================================

/// 偏置向量 (Bias Vector)
/// 定义在环面 T^n = (Z/LZ)^n 上的向量。
/// 所有的运算都必须是在模 L 意义下的。
#[derive(Debug, Clone, Hash)]
pub struct BiasVector {
    pub data: Vec<i32>, // 存储值范围应始终在 [0, BIAS_RING_SIZE)
    // 审计承诺 (Commitment Hash)，用于 ProofBundle
    #[hash(skip)] // 不参与自身的哈希计算，避免循环
    pub commitment: Option<String>,
}

impl BiasVector {
    pub fn new() -> Self {
        // 初始化为零偏置
        BiasVector {
            data: vec![0; BIAS_DIM],
            commitment: None,
        }
    }

    /// 计算并锁定该 Bias 的承诺 (Commitment)
    /// 这对应于 "GlobalRoot_bias" 的生成过程
    pub fn seal(&mut self) -> String {
        let mut hasher = DefaultHasher::new();
        self.data.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());
        self.commitment = Some(hash.clone());
        hash
    }

    /// 在环面上计算两个 Bias 向量之间的“曼哈顿环面距离”
    /// d(x, y) = sum( min(|x_i - y_i|, L - |x_i - y_i|) )
    /// 这是验证 Lipschitz 稳定性的关键指标。
    pub fn torus_distance(&self, other: &BiasVector) -> i32 {
        self.data
            .iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| {
                let raw_diff = (a - b).abs();
                std::cmp::min(raw_diff, BIAS_RING_SIZE - raw_diff)
            })
            .sum()
    }

    /// 应用扰动 (Perturbation) 并保持在环面上 (Wrap-around)
    /// val_new = (val_old + delta) mod L
    pub fn apply_perturbation(&mut self, idx: usize, delta: i32) {
        if idx < self.data.len() {
            // 使用 rem_euclid 确保结果总是正数 [0, L)
            self.data[idx] = (self.data[idx] + delta).rem_euclid(BIAS_RING_SIZE);
        }
    }

    /// 使用投影矩阵将环面 Bias 映射到 Logits 线性空间
    ///
    /// # Cyclic Embedding & Dense Projection
    /// 1. 循环嵌入: b_i -> [sin(theta), cos(theta)]，消除 0/L 边界断裂。
    /// 2. 密集投影: 每个 Action logit 都是所有 Bias 维度的加权和。
    /// 这种组合确保了对动作空间的细粒度控制 (Fine-grained Control)。
    pub fn project_to_logits_with(&self, proj: &BiasProjector) -> Vec<f64> {
        let mut phi = vec![0.0; 2 * BIAS_DIM];

        // 1. Cyclic Embedding
        for (i, &val) in self.data.iter().enumerate() {
            // 计算角度 theta = 2 * pi * val / L
            let theta = 2.0 * PI * (val as f64) / (BIAS_RING_SIZE as f64);
            phi[2 * i] = theta.sin();
            phi[2 * i + 1] = theta.cos();
        }

        // 2. Dense Matrix Multiplication
        let mut out = vec![0.0; ACTION_SPACE_SIZE];
        for k in 0..ACTION_SPACE_SIZE {
            let mut s = 0.0;
            // 每一个 Action k 都受到所有 Bias 维度的影响
            for j in 0..2 * BIAS_DIM {
                s += proj.w[k][j] * phi[j];
            }
            out[k] = proj.scale * s;
        }
        out
    }
}

// =========================================================================
// Controller & Optimization
// =========================================================================

/// 审计记录 (Audit Record)
/// 记录每一次修正的快照，用于事后验证 (Proof of Logic)
#[derive(Debug, Clone)]
pub struct BiasAuditRecord {
    pub timestamp: u64,      // 逻辑时钟
    pub commitment: String,  // Bias Hash
    pub bias_snapshot: Vec<i32>,
    pub final_energy: f64,
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
    projector: BiasProjector, // 持有投影器实例
    config: VapoConfig,
    // 审计日志：存储所有的 ProofBundle
    pub audit_log: Vec<BiasAuditRecord>,
}

impl BiasController {
    pub fn new(config: Option<VapoConfig>) -> Self {
        BiasController {
            current_bias: BiasVector::new(),
            projector: BiasProjector::new(), // 初始化投影器
            config: config.unwrap_or_default(),
            audit_log: Vec::new(),
        }
    }

    /// VAPO 核心循环：搜索最优 Bias 以最小化 STP 能量
    ///
    /// # 参数
    /// - `base_logits`: 生成器原始输出的 Logits
    /// - `stp_ctx`: 代数状态上下文
    /// - `decode_fn`: Logits -> ProofAction 解码器
    ///
    /// # 返回
    /// - `BiasVector`: 修正后并已 Seal 的偏置向量
    /// - `ProofAction`: 修正后的动作
    pub fn optimize<F>(
        &mut self,
        base_logits: &[f64],
        stp_ctx: &STPContext,
        decode_fn: F,
    ) -> (BiasVector, ProofAction)
    where
        F: Fn(&[f64]) -> ProofAction,
    {
        // -----------------------------------------------------------------
        // Phase 1: Fast Path (System 1 - Intuition)
        // -----------------------------------------------------------------
        // 首先检查：Generator 的原始直觉是否已经正确？
        // 如果正确 (Energy == 0)，直接放行，不进入 VAPO 循环。
        // 这极大地降低了推理延迟。
        let initial_action = decode_fn(base_logits);
        let initial_energy = stp_ctx.calculate_energy(&initial_action);

        if initial_energy <= 1e-6 {
            // Latency Optimization: Skip VAPO!
            // 为了保持信号纯净，我们在这里使用一个全新的 Zero Bias，
            // 并更新 controller 状态。
            let mut zero_bias = BiasVector::new();
            self.record_artifact(&mut zero_bias, initial_energy);
            self.current_bias = zero_bias.clone();
            return (zero_bias, initial_action);
        }

        // -----------------------------------------------------------------
        // Phase 2: VAPO Loop (System 2 - Reasoning)
        // -----------------------------------------------------------------
        // Generator 犯错或者不够完美，启动 VAPO 引擎进行修正。

        let mut best_bias = self.current_bias.clone();
        // 基于当前的 bias 起点进行计算 (保持控制连续性)
        let start_logits = self.apply_bias(base_logits, &best_bias);
        let mut best_action = decode_fn(&start_logits);
        let mut min_energy = stp_ctx.calculate_energy(&best_action);

        // Deduplication Cache: Action -> Energy
        // 很多微小的 Bias 扰动不会改变 Argmax 出来的离散动作。
        // 我们缓存动作的能量计算结果，避免昂贵的 STP 上下文查询。
        let mut energy_cache: HashMap<ProofAction, f64> = HashMap::new();
        energy_cache.insert(best_action.clone(), min_energy);

        // 如果之前的 Bias 依然有效，也直接返回
        if min_energy <= 1e-6 {
            self.record_artifact(&mut best_bias, min_energy);
            return (best_bias, best_action);
        }

        let mut rng = rand::thread_rng();
        let mut temperature = self.config.initial_temperature;

        // VAPO 搜索循环
        for _iter in 0..self.config.max_iterations {
            // 1. 生成扰动 (Perturbation)
            let mut candidate_bias = best_bias.clone();
            let dim_idx = rng.gen_range(0..BIAS_DIM);

            // Valuation-Adaptive: 能量越大，扰动越剧烈
            let perturbation_strength = if min_energy > 1.0 {
                rng.gen_range(-10..=10) // 粗调 (Coarse Tuning)
            } else {
                rng.gen_range(-2..=2) // 微调 (Fine Tuning)
            };

            candidate_bias.apply_perturbation(dim_idx, perturbation_strength);

            // 2. 应用 Bias 并解码 (With Dense Projection)
            let modified_logits = self.apply_bias(base_logits, &candidate_bias);
            let candidate_action = decode_fn(&modified_logits);

            // 3. 计算新能量 (With Cache)
            let new_energy = if let Some(&e) = energy_cache.get(&candidate_action) {
                e // Cache Hit
            } else {
                let e = stp_ctx.calculate_energy(&candidate_action);
                energy_cache.insert(candidate_action.clone(), e);
                e
            };

            // 4. Metropolis-Hastings 接受准则
            let delta_e = new_energy - min_energy;
            if delta_e < 0.0 || rng.gen::<f64>() < (-delta_e / temperature).exp() {
                best_bias = candidate_bias;
                min_energy = new_energy;
                best_action = candidate_action;

                if min_energy <= 1e-6 {
                    break;
                }
            }

            temperature *= self.config.valuation_decay;
        }

        // 5. 记录审计产物
        self.record_artifact(&mut best_bias, min_energy);

        // 更新内部状态
        self.current_bias = best_bias.clone();

        (best_bias, best_action)
    }

    /// 内部方法：Seal bias 并写入审计日志
    fn record_artifact(&mut self, bias: &mut BiasVector, energy: f64) {
        let commitment = bias.seal();
        self.audit_log.push(BiasAuditRecord {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            commitment,
            bias_snapshot: bias.data.clone(),
            final_energy: energy,
        });
    }

    /// 将 Bias 叠加到 Base Logits 上
    fn apply_bias(&self, base: &[f64], bias: &BiasVector) -> Vec<f64> {
        // 使用 Projector 进行映射
        let bias_proj = bias.project_to_logits_with(&self.projector);
        base.iter()
            .zip(bias_proj.iter())
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
    fn test_bias_ring_homomorphism() {
        let mut b1 = BiasVector::new();
        b1.apply_perturbation(0, -1);
        assert_eq!(b1.data[0], BIAS_RING_SIZE - 1);

        let mut b2 = BiasVector::new();
        b2.apply_perturbation(0, 1);
        assert_eq!(b2.data[0], 1);

        let dist = b1.torus_distance(&b2);
        assert_eq!(dist, 2, "Torus distance failed!");
    }

    #[test]
    fn test_fast_path_optimization() {
        // 测试当 Generator 正确时，VAPO 是否能快速跳过
        let mut stp_ctx = STPContext::new();
        let mut controller = BiasController::new(None);

        // 模拟一个"完美"的 Logits
        let correct_logits = vec![100.0; ACTION_SPACE_SIZE];

        // 简单的 Mock Decoder
        let decode_fn = |_: &[f64]| -> ProofAction {
            ProofAction::QED // QED energy is always 0.0
        };

        let (final_bias, _) = controller.optimize(&correct_logits, &stp_ctx, decode_fn);

        // 验证结果是 Zero Bias (因为走了 Fast Path)
        assert_eq!(final_bias.data.iter().sum::<i32>(), 0);
        // 验证审计日志里有记录
        assert_eq!(controller.audit_log.len(), 1);
        assert!(controller.audit_log[0].final_energy <= 1e-6);
    }
}
