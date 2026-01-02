use serde::{Deserialize, Serialize};
use crate::soul::algebra::IdealClass;
use crate::will::perturber::Perturber;

pub type Energy = f64;

/// 验证结果
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    Verified { energy: Energy, steps: usize },
    InvalidUniverse { details: String },
    IllegalMove { step: usize, generator: String },
    ContextMismatch { expected_seed: String, actual_seed: String },
    FinalStateMismatch { claimed: String, calculated: String },
    EnergyMismatch { claimed: Energy, calculated: Energy },
}

/// 优化轨迹 (Proof of Will Certificate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationTrace {
    pub id: String,
    pub timestamp: u64,
    pub context: String, // [New] 必须包含 Context 以验证种子来源
    
    /// 初始种子 (S_0)
    pub initial_state: IdealClass,
    
    /// 扰动序列 (u_0, u_1, ..., u_k)
    pub perturbations: Vec<IdealClass>,
    
    /// 最终状态 (S_final)
    pub final_state: IdealClass,
    
    pub claimed_energy: Energy,
}

impl OptimizationTrace {
    pub fn new(initial_state: IdealClass, context: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            context,
            initial_state: initial_state.clone(),
            perturbations: Vec::new(),
            final_state: initial_state, 
            claimed_energy: f64::MAX,
        }
    }

    pub fn record_step(&mut self, perturbation: IdealClass) {
        self.final_state = self.final_state.compose(&perturbation);
        self.perturbations.push(perturbation);
    }

    pub fn finalize(&mut self, final_energy: Energy) {
        self.claimed_energy = final_energy;
    }
}

/// 验证器 (The Verifier)
pub struct TraceVerifier;

impl TraceVerifier {
    /// 严格验证流程
    /// 1. Anchor Check: 验证 initial_state 是否由 context 确定性生成
    /// 2. Graph Check: 验证每一步是否在允许的生成元集合 P 中
    /// 3. Algebra Check: 重放群运算，确保宇宙一致性
    /// 4. Energy Check: 审计最终能量
    pub fn verify<E>(
        trace: &OptimizationTrace, 
        energy_fn: E,
        perturbation_count: usize // 用于重建 P 集合
    ) -> VerificationResult
    where 
        E: Fn(&IdealClass) -> Energy,
    {
        // --- 1. Anchor Check (Proof of Search Context) ---
        // 攻击者不能随便拿一个 S_0 来跑，必须证明 S_0 源自这个 Context。
        // 由于 IdealClass::from_hash 包含了复杂的素数搜索，这一步验证了 "Puzzle Input"。
        
        // 注意：这里为了演示，传入 p=0。实际生产中 p 应从 system parameters 获取或包含在 trace header 中
        // 假设 lib.rs 中的 PyEvolver 默认 p=409 (或其他值)，这里需要对齐。
        // 暂时假设 p 不影响代数结构（只影响投影），所以 IdealClass::from_hash 的第二个参数
        // 实际上只在 Projector 里用到，但 IdealClass 初始化也需要一个占位符。
        // 我们直接调用 IdealClass::spawn_universe 获取纯净的代数种子。
        
        let (expected_seed, _) = IdealClass::spawn_universe(&trace.context);
        
        if expected_seed != trace.initial_state {
            return VerificationResult::ContextMismatch {
                expected_seed: format!("{}", expected_seed),
                actual_seed: format!("{}", trace.initial_state),
            };
        }

        // --- 2. Graph Topology Setup (Reconstruct P) ---
        // 验证者必须独立重建生成元集合，不能信任 Trace 里提供的任何元数据
        let discriminant = trace.initial_state.discriminant();
        let perturber = Perturber::new(&discriminant, perturbation_count);
        let allowed_generators = perturber.get_generators(); // 需要 Perturber 公开此方法

        // --- 3. Path Replay (Graph & Algebra Check) ---
        let mut calculated_state = trace.initial_state.clone();
        
        for (i, u) in trace.perturbations.iter().enumerate() {
            // A. 检查 u 是否在 P 或 P^-1 中
            let is_valid_generator = allowed_generators.contains(u);
            let is_valid_inverse = if !is_valid_generator {
                let inverse = u.inverse();
                allowed_generators.contains(&inverse)
            } else {
                true
            };

            if !is_valid_generator && !is_valid_inverse {
                return VerificationResult::IllegalMove { 
                    step: i, 
                    generator: format!("{}", u) 
                };
            }

            // B. 执行群运算 (compose 内部现在会 Panic 如果宇宙不一致，
            // 但为了更友好的错误处理，我们可以在这里捕获 panic，或者依赖前面的 ensure_universe)
            // 由于我们是在 Rust 中，compose panic 会导致 crash。
            // 理想情况下 compose 应该返回 Result，我们在 algebra.rs 里加了 ensure_same_universe。
            // 这里我们显式检查一下以防万一。
            if let Err(e) = calculated_state.ensure_same_universe(u) {
                return VerificationResult::InvalidUniverse { details: e };
            }

            calculated_state = calculated_state.compose(u);
        }

        // --- 4. Final Consistency Check ---
        if calculated_state != trace.final_state {
            return VerificationResult::FinalStateMismatch {
                claimed: format!("{}", trace.final_state),
                calculated: format!("{}", calculated_state),
            };
        }

        // --- 5. Energy Audit ---
        let calculated_energy = energy_fn(&calculated_state);
        let epsilon = 1e-6;
        if (calculated_energy - trace.claimed_energy).abs() > epsilon {
            return VerificationResult::EnergyMismatch { 
                claimed: trace.claimed_energy, 
                calculated: calculated_energy 
            };
        }

        VerificationResult::Verified { 
            energy: calculated_energy, 
            steps: trace.perturbations.len() 
        }
    }
}
