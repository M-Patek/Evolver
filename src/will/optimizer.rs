use crate::soul::algebra::{IdealClass, Quaternion};
use crate::will::evaluator::Evaluator;
use crate::will::perturber::{Perturber, HeckePerturber};
use crate::will::tracer::Trace;

/// VAPO 优化器 (Graph Walker 版)
/// 
/// 在 Ramanujan 图上执行估值自适应扰动优化。
/// 由于新架构中使用了定四元数算术格，图具有最优的谱隙 (Spectral Gap)。
/// 这意味着随机游走或贪婪搜索能以极快的速度混合，避免陷入局部最优。
pub struct VapoOptimizer {
    evaluator: Box<dyn Evaluator>,
    perturber: Box<dyn Perturber>,
    max_steps: usize,
}

impl VapoOptimizer {
    /// 创建一个新的优化器实例
    pub fn new(evaluator: Box<dyn Evaluator>, max_steps: usize) -> Self {
        Self {
            evaluator,
            perturber: Box::new(HeckePerturber::new()), // 默认使用 Hecke 扰动
            max_steps,
        }
    }

    /// 执行进化搜索
    /// 输入种子状态，返回能量最低的状态及其因果路径 (Trace)。
    pub fn search(&self, seed: &IdealClass) -> (IdealClass, Trace) {
        let mut current_state = seed.clone();
        let mut current_energy = self.evaluator.evaluate(&current_state);
        let mut trace = Trace::new();

        // 初始记录
        trace.record(Quaternion::identity(), current_energy);

        for step in 0..self.max_steps {
            // 1. 如果能量为 0，真理已找到，立即停止。
            if current_energy <= 1e-6 {
                println!("喵！在第 {} 步找到了真理 (Energy=0)！", step);
                break;
            }

            // 2. 获取所有可能的因果移动 (Hecke Neighbors)
            let moves = self.perturber.get_moves();
            
            // 3. 贪婪评估 (Look-ahead 1 step)
            // 在 Ramanujan 图上，邻域包含了足够的信息来指导下降。
            let mut best_move = Quaternion::identity();
            let mut best_energy = f64::MAX;
            let mut found_better = false;

            for mv in &moves {
                // S_next = S_curr * Operator
                let next_state = current_state.apply_hecke(mv);
                let energy = self.evaluator.evaluate(&next_state);

                if energy < best_energy {
                    best_energy = energy;
                    best_move = *mv;
                    found_better = true;
                }
            }

            // 4. 状态转移决策
            // 这里使用了简单的最速下降法 (Steepest Descent)。
            // 在更复杂的版本中，可以引入 Metropolis-Hastings 准则来允许以一定概率接受坏状态（热涨落）。
            if found_better && best_energy < current_energy {
                current_state = current_state.apply_hecke(&best_move);
                current_energy = best_energy;
                
                // 记录因果算子
                trace.record(best_move, current_energy);
            } else {
                // 如果陷入局部最优 (虽然在 Ramanujan 图上很少见)，
                // 我们可以选择随机跳跃或者通过 "Identity" 保持不动并终止。
                // 这里为了鲁棒性，我们尝试一个随机扰动来打破僵局。
                // (简化处理：直接停止或继续搜索)
                // println!("陷入局部最优，能量: {}", current_energy);
            }
        }

        (current_state, trace)
    }
}
