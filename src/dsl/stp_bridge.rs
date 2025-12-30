// src/dsl/stp_bridge.rs
use std::collections::HashMap;
use crate::dsl::schema::ProofAction;
use crate::crypto::algebra::Matrix; 

/// STP 上下文环境
/// 负责维护当前的代数状态，并将高层的 ProofAction 编译为底层的矩阵算子。
pub struct STPContext {
    // 状态矩阵：代表当前系统的全局状态 x(t)
    pub state_matrix: Matrix,
    
    // 能量值：E(t)，0.0 表示自洽，>0 表示存在矛盾
    pub energy: f64,
    
    // 符号表：存储变量名及其对应的代数定义 (Valuation)
    // 在 Mock 阶段，我们用简单的 Label 矩阵来表示 (Odd/Even)
    pub variables: HashMap<String, Matrix>,
}

impl STPContext {
    pub fn new() -> Self {
        STPContext {
            state_matrix: Matrix::identity(1),
            energy: 0.0,
            variables: HashMap::new(),
        }
    }

    /// 核心接口：计算给定动作的能量
    /// 注意：这里必须是 &mut self，因为计算过程可能会更新内部缓存或临时状态
    pub fn calculate_energy(&mut self, action: &ProofAction) -> f64 {
        // 1. 预演动作 (Dry Run)
        // 将 Action 转换为对状态的预期影响
        let energy = match action {
            ProofAction::Define { symbol, hierarchy_path } => {
                // 定义动作：通常总是合法的，除非重定义冲突
                // 在这个 Mock 实现中，我们简单地注册变量
                let val = self.mock_valuation_from_path(hierarchy_path);
                self.variables.insert(symbol.clone(), val);
                0.0
            },
            
            ProofAction::Apply { theorem_id, inputs, output_symbol } => {
                // 推导动作：检查 inputs 结合 theorem 是否能推导出 output_symbol
                self.check_inference_consistency(theorem_id, inputs, output_symbol)
            },

            // 其他动作暂时返回 0 (Pass)
            _ => 0.0,
        };

        self.energy = energy;
        energy
    }

    // --- 内部辅助逻辑 ---

    /// 模拟：根据路径生成一个代数值
    fn mock_valuation_from_path(&self, path: &[String]) -> Matrix {
        // 简单 Mock：如果是 "Odd" 返回 [1, 0], "Even" 返回 [0, 1]
        if path.contains(&"Odd".to_string()) {
            Matrix::new(vec![vec![1.0], vec![0.0]]) // Vector [1, 0]
        } else {
            Matrix::new(vec![vec![0.0], vec![1.0]]) // Vector [0, 1]
        }
    }

    /// 检查推导一致性 (Energy Function)
    fn check_inference_consistency(&self, theorem: &str, inputs: &[String], output_sym: &str) -> f64 {
        if theorem == "ModAdd" && inputs.len() == 2 {
            let val_a = self.variables.get(&inputs[0]);
            let val_b = self.variables.get(&inputs[1]);
            let val_out = self.variables.get(output_sym); // 注意：这里假设 Output 已经被 Generator 尝试定义了

            if let (Some(a), Some(b), Some(out)) = (val_a, val_b, val_out) {
                // 逻辑：Odd(1,0) + Odd(1,0) should be Even(0,1)
                // 简单的矩阵加法模拟 STP 乘法逻辑
                let is_a_odd = a[0][0] > 0.5;
                let is_b_odd = b[0][0] > 0.5;
                let is_out_odd = out[0][0] > 0.5;

                let expected_odd = is_a_odd ^ is_b_odd; // XOR: Odd+Odd=Even(False), Odd+Even=Odd(True)
                
                if is_out_odd == expected_odd {
                    return 0.0; // 符合逻辑
                } else {
                    return 10.0; // 逻辑矛盾！能量激增
                }
            }
        }
        0.0 // 默认宽容
    }
}
