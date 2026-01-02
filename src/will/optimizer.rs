use crate::soul::algebra::IdealClass;
use crate::will::evaluator::Evaluator;
use crate::will::tracer::OptimizationTrace;
use crate::will::perturber::Perturber;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// 软屏障能量：当遇到代数错误时，不Crash，而是返回这个能量值
const SOFT_BARRIER_ENERGY: f64 = 1.0e9;

pub struct VapoOptimizer {
    evaluator: Box<dyn Evaluator>,
    max_steps: usize,
}

impl VapoOptimizer {
    pub fn new(evaluator: Box<dyn Evaluator>, max_steps: usize) -> Self {
        Self {
            evaluator,
            max_steps,
        }
    }

    /// 执行 VAPO 搜索
    /// 返回 (最优状态, 搜索轨迹)
    pub fn search(&self, initial_state: &IdealClass) -> (IdealClass, OptimizationTrace) {
        let mut current_state = initial_state.clone();
        let mut current_energy = self.evaluator.evaluate(&current_state);
        
        let discriminant = initial_state.discriminant();
        let mut trace = OptimizationTrace::new(initial_state.clone(), "unknown_context".to_string());
        trace.finalize(current_energy); // Init

        // 初始化扰动器 (Generators)
        // 假设 k=10 (perturbation count)
        let perturber = Perturber::new(&discriminant, 10);
        let generators = perturber.get_generators();

        let mut rng = thread_rng();

        for _step in 0..self.max_steps {
            // 1. 终止条件
            if current_energy < 1e-6 {
                break;
            }

            // 2. 邻域采样 (Sampling Neighborhood)
            // 尝试随机选取一个生成元进行移动
            if let Some(perturbation) = generators.choose(&mut rng) {
                // 3. 试探性移动 (Trial Move)
                // [Robustness] 这里使用了 Result 处理，捕获任何代数异常
                let trial_result = current_state.compose(perturbation);

                match trial_result {
                    Ok(trial_state) => {
                        let trial_energy = self.evaluator.evaluate(&trial_state);

                        // 4. 贪婪接受准则 (Greedy Acceptance)
                        // 实际 VAPO 可能会用 Metropolis，这里简化为贪婪
                        if trial_energy < current_energy {
                            current_state = trial_state;
                            current_energy = trial_energy;
                            
                            // 记录到 Trace
                            trace.record_step(perturbation.clone());
                            trace.finalize(current_energy);
                        }
                    },
                    Err(e) => {
                        // [Soft Barrier Logic]
                        // 遇到代数错误（比如宇宙不匹配，虽然这里不太可能因为 generators 都是合法的），
                        // 我们将其视为撞到了“高能量墙”。
                        // 不做任何状态更新，相当于这一步被 Reject 了。
                        eprintln!("VAPO hit a barrier: {}. Retrying...", e);
                        // 可以在这里增加某种惩罚机制，或者调整 temperature
                    }
                }
            }
        }

        (current_state, trace)
    }
}
