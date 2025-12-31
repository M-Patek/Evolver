use pyo3::prelude::*;
use crate::soul::algebra::ClassGroupElement;
use crate::will::optimizer::VapoOptimizer;
use crate::body::decoder::BodyProjector;
use crate::body::adapter::SemanticAdapter;
use crate::dsl::stp_bridge::STPContext;

pub mod soul;
pub mod will;
pub mod body;
pub mod dsl;

/// Python 模块入口
#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    Ok(())
}

/// Evolver 引擎的 Python 包装类
///
/// 这是系统的主控制器，负责协调 Soul (代数), Will (优化), Body (投影) 和 Logic (验证) 的工作流。
#[pyclass]
struct PyEvolver {
    // 判别式参数 P (用于代数群构造)
    p: u64,
    // 投影参数 K (用于控制投影维度/路径长度)
    k: usize,
}

#[pymethods]
impl PyEvolver {
    #[new]
    fn new(p: u64, k: usize) -> Self {
        PyEvolver { p, k }
    }

    /// 对齐逻辑 (Align Logic)
    ///
    /// 执行核心的演化-优化循环。
    /// 1. 从上下文中通过 Hash 生成代数种子。
    /// 2. 使用 VAPO 算法在凯莱图上搜索。
    /// 3. 将代数状态投影为数字序列，再物化为逻辑动作。
    /// 4. 通过 STP 引擎计算逻辑违规能量。
    /// 5. 最小化能量，直到找到真理 (Energy = 0)。
    ///
    /// # 参数
    /// * `context`: 用户输入的自然语言上下文 (字符串)
    ///
    /// # 返回
    /// * `Vec<String>`: 生成的逻辑证明步骤列表
    fn align(&self, context: String) -> PyResult<Vec<String>> {
        // --- 1. Inception (起源) ---
        // 初始化代数种子 (Soul)
        let seed = ClassGroupElement::from_context(&context, self.p);
        
        // --- 2. The Will (意志) ---
        // 初始化 VAPO 优化器，赋予它初始状态
        let mut optimizer = VapoOptimizer::new(seed);
        
        let mut best_energy = f64::MAX;
        let mut best_path_digits = Vec::new();
        let mut best_actions = Vec::new(); 
        
        // 搜索参数
        let max_iterations = 1000;

        // --- 3. Evolution Loop (演化循环) ---
        for _ in 0..max_iterations {
            // A. Perturbation (扰动/搜索)
            // 意志在图上移动一步，寻找新的候选状态
            let current_state = optimizer.perturb();
            
            // B. Materialization (物化/时间展开)
            // 根据最新的规范，这里隐含了时间维度的展开 (Time Evolution)。
            // BodyProjector 将代数状态投影为一串数字。
            let path_digits = BodyProjector::project(&current_state, self.k, self.p);

            // C. Semantic Adaptation (语义适配)
            // [关键修复]：整个数字序列被转换为逻辑动作，不再是仅取首位。
            let logic_actions = SemanticAdapter::materialize(&path_digits);

            // D. Judgement (审判/能量计算)
            // STP 引擎计算这组逻辑动作的自洽性。
            let mut stp = STPContext::new();
            let energy = stp.calculate_energy(&logic_actions);

            // E. Selection (自然选择)
            // 如果新状态的能量更低（更接近真理），或者基于模拟退火概率接受
            if energy < best_energy {
                best_energy = energy;
                best_path_digits = path_digits.clone();
                best_actions = logic_actions.clone();
                
                // 告诉优化器接受这个位置作为新的搜索基点
                optimizer.accept(current_state.clone());
            }

            // F. Convergence (收敛)
            // 如果能量为 0，说明找到了绝对真理，停止搜索。
            if best_energy == 0.0 {
                break;
            }
        }

        // --- 4. Revelation (启示/输出) ---
        
        // 如果搜索失败 (能量仍 > 0)，我们可以选择返回错误或返回“最可信”的路径。
        // 这里返回找到的最佳路径，并在开头加上能量标记以便调试。
        let mut result_strings: Vec<String> = Vec::new();
        
        // (可选) 添加元数据
        result_strings.push(format!("# Final Energy: {:.1}", best_energy));
        result_strings.push(format!("# Algebraic Path: {:?}", best_path_digits));

        // 添加具体的逻辑步骤
        for action in best_actions {
            result_strings.push(format!("{:?}", action));
        }

        Ok(result_strings)
    }
}
