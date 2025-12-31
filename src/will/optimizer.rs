use num_bigint::BigInt;
use num_traits::{Zero, Signed};
use crate::soul::algebra::ClassGroupElement;
use crate::will::perturber::{self, EnergyEvaluator};

/// VAPO (Valuation-Adaptive Perturbation Optimization) 核心循环
///
/// 该函数执行局部离散搜索（Hill Climbing / Metropolis-Hastings 的变体）。
/// 
/// # 逻辑流程
/// 1. 从 `start_state` 提取判别式 $\Delta$。
/// 2. 生成一批微小的代数扰动 $\{\epsilon_i\}$。
/// 3. 进入优化循环：
///    - **估值调度 (Valuation Schedule)**: 随着迭代进行，动态调整扰动窗口。
///      初期允许“巨大”扰动以跳出深坑，后期收缩至“微小”扰动进行精细对齐。
///    - 对当前状态应用有效窗口内的扰动，生成候选集。
///    - 将候选状态“具象化”为路径（Digits），评估其 STP 能量。
///    - 贪婪地选择能量最低的状态作为下一次迭代的起点。
///    - 如果发现能量 $E=0$ 的状态，立即返回（Bingo!）。
/// 4. 如果超过最大迭代次数仍未收敛，返回当前找到的最好的状态（Best Effort）。
pub fn optimize(
    start_state: &ClassGroupElement,
    evaluator: &impl EnergyEvaluator
) -> ClassGroupElement {
    // 1. 自动提取判别式: Delta = b^2 - 4ac
    // 这是一个不变量，定义了我们所在的类群。
    let four = BigInt::from(4);
    let delta = (&start_state.b * &start_state.b) - (&four * &start_state.a * &start_state.c);

    // 2. 准备扰动集 (The Perturbation Set)
    // 我们生成前 50 个分裂素数对应的微小群元素。
    // 数量增加以支持初期的“大幅度”探索（大素数对应更大的群结构跳跃）。
    let perturbation_count = 50;
    let perturbations = perturber::generate_perturbations(&delta, perturbation_count);

    let mut current_state = start_state.clone();
    let mut current_energy = evaluate_state(&current_state, evaluator);
    
    // 如果初始状态就是完美的，直接返回
    if current_energy.abs() < 1e-6 {
        return current_state;
    }

    let max_iterations = 100;

    // 3. 优化循环 (The Will Loop)
    for _iter in 0..max_iterations {
        // [Task 3.4] 估值自适应调度 (Valuation Schedule)
        // -------------------------------------------------------------
        // 逻辑：模拟退火 / 由粗到精 (Coarse-to-Fine)
        // - 进度 (Progress): 0.0 -> 1.0
        // - 窗口 (Window): 覆盖所有 50 个扰动 -> 收缩至前 5 个扰动
        //
        // 原理：列表中的扰动是按范数（素数 p）升序排列的。
        // "大范围"搜索意味着我们要利用列表尾部较大的 p 进行大幅度状态跳跃。
        // "精细"搜索意味着我们只使用列表头部极小的 p (如 p=2, 3) 进行微调。
        let progress = _iter as f64 / max_iterations as f64;
        
        // 衰减函数：线性收缩
        // start_ratio = 1.0 (使用 100% 的扰动)
        // end_ratio = 0.1 (只使用 10% 的最小扰动)
        let window_ratio = 1.0 - (0.9 * progress); 
        let active_count = (perturbations.len() as f64 * window_ratio).ceil() as usize;
        
        // 守卫：至少保留 3 个最小扰动，防止窗口过于狭窄导致死锁
        let active_count = active_count.max(3).min(perturbations.len());
        
        let active_perturbations = &perturbations[0..active_count];
        // -------------------------------------------------------------

        let mut best_candidate = current_state.clone();
        let mut min_energy = current_energy;
        let mut found_better = false;

        // 并行评估所有候选者 (这里简化为串行，实际部署建议用 Rayon)
        for eps in active_perturbations {
            // 正向扰动: S' = S * eps
            let candidate_pos = current_state.compose(eps);
            let energy_pos = evaluate_state(&candidate_pos, evaluator);

            if energy_pos < min_energy {
                min_energy = energy_pos;
                best_candidate = candidate_pos;
                found_better = true;
            }

            // 逆向扰动: S' = S * eps^-1 (利用逆元进行双向搜索)
            // 注：ClassGroupElement 的逆元通常是 (a, -b, c)
            let inverse_eps = eps.inverse();
            let candidate_neg = current_state.compose(&inverse_eps);
            let energy_neg = evaluate_state(&candidate_neg, evaluator);

            if energy_neg < min_energy {
                min_energy = energy_neg;
                best_candidate = candidate_neg; // 修正逻辑：更新 best_candidate
                found_better = true;
            }
        }

        // 决策时刻
        if min_energy.abs() < 1e-6 {
            // 找到了完美解！
            return best_candidate;
        }

        if found_better {
            // 登山：移动到能量更低的状态
            current_state = best_candidate;
            current_energy = min_energy;
        } else {
            // 局部最优陷阱 (Local Minima)
            // 随着窗口缩小，如果在这个精细度下找不到更好的解，我们选择终止，
            // 而不是盲目地继续（因为再迭代窗口只会更小）。
            // 
            // 在更复杂的实现中，这里可以加入 "Reheat" (重热) 机制，
            // 暂时放大窗口尝试跳出，但目前保持简单。
            break;
        }
    }

    current_state
}

/// 辅助函数：将代数状态具象化并评估能量
/// 
/// "Materialize Path": 将抽象的代数对象 $(a, b, c)$ 投影到
/// 物理引擎可以理解的数字序列（Digits/Tokens）。
fn evaluate_state(state: &ClassGroupElement, evaluator: &impl EnergyEvaluator) -> f64 {
    let path = materialize_path(state);
    evaluator.evaluate(&path)
}

/// 具象化路径 (Materialize Path)
///
/// 将 ClassGroupElement 转换为 u64 序列。
/// 这是一个从“理型世界”到“现实世界”的投影。
/// 
/// 这里的实现是一个简单的哈希投影，实际系统中会连接到 `src/body/projection.rs`
/// 使用投影矩阵 $W$。
fn materialize_path(state: &ClassGroupElement) -> Vec<u64> {
    // 临时逻辑：将 (a, b, c) 的低 64 位作为特征向量
    // 注意：BigInt 到 u64 可能会截断，但这对于简单的指纹足够了
    let mut digits = Vec::new();
    
    // 简单的转换逻辑，避免 unwrap panic
    let extract_u64 = |n: &BigInt| -> u64 {
        let (_sign, bytes) = n.to_bytes_le();
        if bytes.is_empty() {
            0
        } else {
            let mut buf = [0u8; 8];
            let len = std::cmp::min(bytes.len(), 8);
            buf[..len].copy_from_slice(&bytes[..len]);
            u64::from_le_bytes(buf)
        }
    };

    digits.push(extract_u64(&state.a));
    digits.push(extract_u64(&state.b));
    digits.push(extract_u64(&state.c));

    digits
}
