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
/// 
/// [Fix] 对齐了 lib.rs 的调用接口，并修复了能量计算逻辑：
/// Energy = Barrier(Tier) + Residual(Geometry)
pub struct StpEvaluator {
    projector: Projector,
    action_count: usize,
    digits_per_action: usize,
    
    /// [New] 目标特征向量。
    /// 这里的“目标”是指 VAPO 搜索的几何引导方向。
    /// 通常由 Context 的哈希生成，或者是用户指定的意图向量。
    target_features: Vec<f64>,
    
    /// 残差权重
    residual_weight: f64,
}

impl StpEvaluator {
    /// 构造函数 [Fix] 对齐 lib.rs 
    /// 注意：lib.rs 传入 (projector, depth, target_features)
    pub fn new(
        projector: Projector, 
        total_depth: usize, 
        target_features: Vec<f64>
    ) -> Self {
        Self { 
            projector, 
            action_count: total_depth / 3, // 假设每个 action 耗费 3 个 digits
            digits_per_action: 3,
            target_features,
            residual_weight: 0.1, 
        }
    }
}

impl Evaluator for StpEvaluator {
    fn evaluate(&self, state: &IdealClass) -> f64 {
        // --- 1. 计算离散势垒 (Discrete Barrier, Truth) ---
        // 使用 Exact Projection (SHA-256)
        
        let total_digits = self.action_count * self.digits_per_action;
        let mut current_state = state.clone();
        
        let mut exact_path = Vec::with_capacity(total_digits);
        for t in 0..total_digits {
            exact_path.push(self.projector.project_exact(&current_state, t as u64));
            // 状态演化 (Identity dynamics for evaluation path)
            // 假设它是瞬时的投影，或者跟随简单的 squaring。
            current_state = current_state.square();
        }
        
        let actions: Vec<_> = exact_path
            .chunks(self.digits_per_action)
            .map(|chunk| adapter::path_to_proof_action(chunk))
            .collect();

        let mut context = STPContext::new();
        // [Fix] 返回分级后的 Barrier 能量 (0.0, 10.x, 100.x)
        let barrier_energy = context.calculate_barrier(&actions);

        // --- 2. 计算连续残差 (Continuous Residual, Will) ---
        // 使用 Continuous Projection (Geometry)
        // 只有当 Barrier 很高时，Residual 才有指导意义（在同一台阶上区分好坏）
        
        let current_features = self.projector.project_continuous(state);
        
        // 简单的欧氏距离
        let mut residual_dist_sq: f64 = 0.0;
        if self.target_features.len() == current_features.len() {
             residual_dist_sq = current_features.iter()
                .zip(self.target_features.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum();
        }

        // 缩放 Residual，确保它只在台阶内部起作用
        // max residual contribution = 0.99
        let residual_energy = (self.residual_weight * residual_dist_sq).min(0.99);

        // Total J(S)
        barrier_energy + residual_energy
    }

    fn name(&self) -> &'static str {
        "STP(Tiered) + Residual"
    }
}
