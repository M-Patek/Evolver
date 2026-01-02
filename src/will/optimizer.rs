use crate::soul::algebra::IdealClass;
use crate::will::evaluator::Evaluator;
use crate::will::perturber::Perturber;

/// VAPO: Valuation-Adaptive Perturbation Optimization.
/// 
/// 离散优化器，负责在类群的 Cayley 图上寻找能量最低的状态。
/// 它是一个通用的搜索框架，具体的“能量”定义由 `Evaluator` 决定。
pub struct VapoOptimizer {
    evaluator: Box<dyn Evaluator>,
    max_steps: usize,
    perturbation_count: usize, // 每次搜索初始化的生成元数量
}

impl VapoOptimizer {
    /// 创建一个新的 VAPO 优化器
    pub fn new(evaluator: Box<dyn Evaluator>, max_steps: usize) -> Self {
        Self {
            evaluator,
            max_steps,
            perturbation_count: 32, // 默认生成 32 个扰动元 (足以覆盖局部连通性)
        }
    }

    /// 执行搜索 (The Will's Journey)
    /// 
    /// # Arguments
    /// * `start_seed` - 初始代数种子 (Born from Context)
    /// 
    /// # Returns
    /// * `IdealClass` - 找到的局部最优状态
    pub fn search(&self, start_seed: &IdealClass) -> IdealClass {
        let mut current_state = start_seed.clone();
        let mut current_energy = self.evaluator.evaluate(&current_state);

        // 0. 如果初始状态即完美，直接返回
        if current_energy.abs() < 1e-6 {
            return current_state;
        }

        // 1. 初始化扰动器 (Context-Aware Perturber)
        // 扰动器依赖于当前宇宙的判别式 \Delta
        let discriminant = start_seed.discriminant();
        let perturber = Perturber::new(&discriminant, self.perturbation_count);

        // 2. 离散梯度下降 / 爬山算法
        for _step in 0..self.max_steps {
            // A. Generate Neighbor (Perturbation)
            // 在图上随机游走一步
            let candidate = perturber.perturb(&current_state);
            
            // B. Sense Energy (Evaluation)
            // 计算新位置的能量
            let candidate_energy = self.evaluator.evaluate(&candidate);

            // C. Selection (Greedy Strategy)
            // 贪婪接受更好的解
            if candidate_energy < current_energy {
                current_state = candidate;
                current_energy = candidate_energy;

                // Found Truth?
                if current_energy.abs() < 1e-6 {
                    break;
                }
            } else {
                // Future: 这里可以加入模拟退火逻辑 (Metropolis-Hastings) 
                // 以接受一定概率的恶化解来跳出局部最优。
            }
        }

        current_state
    }
}
