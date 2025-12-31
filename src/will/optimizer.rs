use num_bigint::BigInt;
use num_traits::{Zero, Signed};
use crate::soul::algebra::ClassGroupElement;
use crate::will::perturber::{self, EnergyEvaluator};
use crate::body::topology::VPuNNConfig; // [Added] 需要配置信息
use crate::body::decoder; // [Added] 使用统一的解码器

/// VAPO (Valuation-Adaptive Perturbation Optimization) 核心循环
///
/// [Architecture Fix]:
/// 移除了本地硬编码的 `materialize_path`，改为注入 `VPuNNConfig` 并调用 `body::decoder`。
/// 这确保了“Will”(优化器) 看到的风景与“Body”(最终输出) 是完全一致的。
/// 这修复了“投影断裂”问题。
pub fn optimize(
    start_state: &ClassGroupElement,
    config: &VPuNNConfig, // [Added] 注入拓扑配置
    evaluator: &impl EnergyEvaluator
) -> ClassGroupElement {
    // 1. 自动提取判别式
    let four = BigInt::from(4);
    let delta = (&start_state.b * &start_state.b) - (&four * &start_state.a * &start_state.c);

    // 2. 准备生成元集 (The Generator Set)
    let perturbation_count = 50;
    let perturbations = perturber::generate_perturbations(&delta, perturbation_count);

    let mut current_state = start_state.clone();
    // 使用统一的投影逻辑计算能量
    let mut current_energy = evaluate_state(&current_state, config, evaluator);
    
    if current_energy.abs() < 1e-6 {
        return current_state;
    }

    let max_iterations = 100;

    // 3. 搜索循环 (The Graph Walk)
    for _iter in 0..max_iterations {
        // [Task 3.4] 生成元子集调度 (Generator Subset Scheduling)
        // 随着迭代，收缩到 Fine (小范数) 生成元，进行局部 Lipschitz 优化
        let progress = _iter as f64 / max_iterations as f64;
        let window_ratio = 1.0 - (0.9 * progress); 
        let active_count = (perturbations.len() as f64 * window_ratio).ceil() as usize;
        let active_count = active_count.max(3).min(perturbations.len());
        
        let active_perturbations = &perturbations[0..active_count];

        let mut best_neighbor = current_state.clone();
        let mut min_energy = current_energy;
        let mut found_better = false;

        // 遍历邻域
        for eps in active_perturbations {
            // 正向边
            let neighbor_pos = current_state.compose(eps);
            let energy_pos = evaluate_state(&neighbor_pos, config, evaluator);

            if energy_pos < min_energy {
                min_energy = energy_pos;
                best_neighbor = neighbor_pos;
                found_better = true;
            }

            // 逆向边
            let inverse_eps = eps.inverse();
            let neighbor_neg = current_state.compose(&inverse_eps);
            let energy_neg = evaluate_state(&neighbor_neg, config, evaluator);

            if energy_neg < min_energy {
                min_energy = energy_neg;
                best_neighbor = neighbor_neg; 
                found_better = true;
            }
        }

        if min_energy.abs() < 1e-6 {
            return best_neighbor;
        }

        if found_better {
            current_state = best_neighbor;
            current_energy = min_energy;
        } else {
            break;
        }
    }

    current_state
}

/// 辅助函数：使用标准解码器计算状态能量
fn evaluate_state(
    state: &ClassGroupElement, 
    config: &VPuNNConfig, 
    evaluator: &impl EnergyEvaluator
) -> f64 {
    // 调用 body 模块的标准物质化函数，而不是本地的 hack
    let path = decoder::materialize_path(state, config);
    evaluator.evaluate(&path)
}
