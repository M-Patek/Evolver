use crate::soul::algebra::IdealClass;
use crate::body::projection::Projector;
use crate::body::adapter;
use crate::dsl::stp_bridge::STPContext;
use num_traits::ToPrimitive;

pub trait Evaluator {
    fn evaluate(&self, state: &IdealClass) -> f64;
    fn name(&self) -> &'static str;
}

pub struct GeometricEvaluator;
impl Evaluator for GeometricEvaluator {
    fn evaluate(&self, state: &IdealClass) -> f64 {
        state.a.to_f64().unwrap_or(1e10)
    }
    fn name(&self) -> &'static str { "Geometric" }
}

/// STP 评估器 (Rigorous Evaluator)
/// 实现了完整的 Unified Energy Metric:
/// J(S) = E_barrier(Exact) + lambda * E_residual(Continuous)
pub struct StpEvaluator {
    projector: Projector,
    action_count: usize,
    digits_per_action: usize,
    
    /// [New] 目标特征向量 (The Desire)
    /// 优化器将试图寻找代数状态 S，使其几何特征接近此目标。
    /// 这提供了优化所需的“斜率”。
    target_features: Vec<f64>,
    
    /// [New] 残差权重 (Lambda)
    residual_weight: f64,
}

impl StpEvaluator {
    pub fn new(projector: Projector, action_count: usize, target_features: Vec<f64>) -> Self {
        Self { 
            projector, 
            action_count,
            digits_per_action: 3,
            target_features,
            residual_weight: 0.1, // 权重需调节，不能淹没 Barrier 的信号
        }
    }
}

impl Evaluator for StpEvaluator {
    fn evaluate(&self, state: &IdealClass) -> f64 {
        // --- 1. 计算离散势垒 (Discrete Barrier, Truth) ---
        // 这一步非常昂贵且不连续，它定义了“有效性”的悬崖。
        
        let total_digits = self.action_count * self.digits_per_action;
        let mut current_state = state.clone();
        
        let mut exact_path = Vec::with_capacity(total_digits);
        for t in 0..total_digits {
            exact_path.push(self.projector.project_exact(&current_state, t as u64));
            current_state = current_state.square();
        }
        
        let actions: Vec<_> = exact_path
            .chunks(self.digits_per_action)
            .map(|chunk| adapter::path_to_proof_action(chunk))
            .collect();

        let mut context = STPContext::new();
        let barrier_energy = context.calculate_energy(&actions);

        // 如果 Barrier 极高 (逻辑完全崩坏)，我们仍然计算 Residual，
        // 这样 VAPO 即使在错误区域也能知道往哪个方向“爬”能接近目标结构。
        
        // --- 2. 计算连续残差 (Continuous Residual, Will) ---
        // J_residual = || Psi(S) - Target ||^2
        
        let current_features = self.projector.project_continuous(state);
        
        let residual_dist_sq: f64 = current_features.iter()
            .zip(self.target_features.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();

        let residual_energy = self.residual_weight * residual_dist_sq;

        // Total J(S)
        // 注意：Barrier 通常是 0, 10, 100 这种离散值
        // Residual 通常是 0.0 ~ 2.0 这种连续值
        barrier_energy + residual_energy
    }

    fn name(&self) -> &'static str {
        "STP + Geometric Residual"
    }
}
