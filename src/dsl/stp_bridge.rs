use std::collections::HashMap;
use crate::dsl::schema::ProofAction;

// ==========================================
// ⚡ Unified Energy Constants
// ==========================================
// 对应文档中的 alpha (Validity Barrier)
// 必须足够大，以确保任何逻辑错误的状态能量都严格大于 0
const PENALTY_BARRIER: f64 = 100.0;

// 对应文档中的 beta (Guidance Coefficient)
// 用于缩放几何距离的影响
const GUIDANCE_BETA: f64 = 1.0;

pub struct STPContext {
    /// 符号表状态：存储变量名到其值的映射 (例如 "n" -> "Odd")
    pub state: HashMap<String, String>,
}

impl STPContext {
    pub fn new() -> Self {
        STPContext {
            state: HashMap::new(),
        }
    }

    /// 计算逻辑动作的能量
    /// 
    /// 实现公式: J(S) = V(Psi(S)) * [alpha + beta * ||Psi(S) - tau||^2]
    pub fn calculate_energy(&mut self, action: &ProofAction) -> f64 {
        match action {
            // 定义动作：通常仅仅是更新状态，不产生能量（除非重定义冲突）
            ProofAction::Define { symbol, hierarchy_path } => {
                if let Some(val) = hierarchy_path.last() {
                    self.state.insert(symbol.clone(), val.clone());
                }
                0.0
            },

            // 应用定理：这是产生能量（逻辑验证）的核心位置
            ProofAction::Apply { theorem_id, inputs, output_symbol } => {
                if theorem_id == "ModAdd" {
                    self.evaluate_mod_add(inputs, output_symbol)
                } else {
                    // 🚨 安全修复：未知定理视为逻辑错误，返回 Barrier 惩罚！
                    // 防止优化器通过调用不存在的定理来欺骗系统获得 0 能量。
                    PENALTY_BARRIER
                }
            },
            
            // 其他动作暂不产生能量
            _ => 0.0,
        }
    }

    /// 评估 ModAdd (奇偶性加法) 的能量
    /// 
    /// 逻辑规则:
    /// Odd + Odd = Even
    /// Even + Even = Even
    /// Odd + Even = Odd
    fn evaluate_mod_add(&self, inputs: &[String], output_symbol: &str) -> f64 {
        // 1. 获取输入值
        let val1 = self.state.get(inputs.get(0).unwrap_or(&"".to_string())).map(|s| s.as_str()).unwrap_or("Unknown");
        let val2 = self.state.get(inputs.get(1).unwrap_or(&"".to_string())).map(|s| s.as_str()).unwrap_or("Unknown");
        
        // 2. 获取当前 VAPO 猜测的输出值 (The Will's Guess)
        let current_guess = self.state.get(output_symbol).map(|s| s.as_str()).unwrap_or("Unknown");

        // 3. 计算这一步的逻辑真值 (Ground Truth)
        let expected = match (val1, val2) {
            ("Odd", "Odd") => "Even",
            ("Even", "Even") => "Even",
            ("Odd", "Even") | ("Even", "Odd") => "Odd",
            _ => "Unknown", // 输入未定义，无法判断
        };

        // 4. 计算统一能量 (Unified Energy)
        if expected == "Unknown" || current_guess == "Unknown" {
            // 🚨 安全修复：上下文缺失也是一种不可接受的状态，必须给予高惩罚
            // 防止优化器通过删除变量定义来“蒙混过关” (Reward Hacking)。
            // 旧逻辑是返回 10.0，这会导致优化器倾向于制造 Unknown 状态来逃避 100.0 的错误惩罚。
            return PENALTY_BARRIER; 
        }

        if current_guess == expected {
            // ✅ Case 1: 逻辑正确 (Truth)
            // J(S) = 0
            return 0.0;
        } else {
            // ❌ Case 2: 逻辑错误 (Violation)
            // J(S) = Barrier + Residual
            // 我们需要计算 "Odd" 和 "Even" 之间的几何距离。
            // 在简单的二元空间中，距离是固定的，但在更复杂的空间中这会有梯度。
            // 这里我们模拟一个距离平方: dist_sq
            
            let dist_sq = self.calculate_semantic_distance(current_guess, expected);
            
            return PENALTY_BARRIER + GUIDANCE_BETA * dist_sq;
        }
    }

    /// 计算语义距离的平方 ||Psi(S) - tau||^2
    /// 这里的实现是一个简化的度量空间
    fn calculate_semantic_distance(&self, s1: &str, s2: &str) -> f64 {
        match (s1, s2) {
            (a, b) if a == b => 0.0,
            
            // Odd 和 Even 是互斥的，距离定义为 1.0
            ("Odd", "Even") | ("Even", "Odd") => 1.0,
            
            // 如果是一个稍微接近的概念 (例如 "Integer" vs "Odd")，距离可以小一点
            ("Integer", "Odd") | ("Odd", "Integer") => 0.5,
            
            // 完全不相关的概念，距离很大
            _ => 5.0,
        }
    }
}
