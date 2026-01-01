// src/dsl/stp_bridge.rs
// STP 桥接器: 消费强类型的 ProofAction 并计算系统能量

use crate::dsl::schema::{ProofAction, LogicType, LogicValue};
use std::collections::HashMap;

pub struct STPContext {
    // 符号表：存储变量名及其当前的逻辑值
    symbol_table: HashMap<String, LogicValue>,
    // 累积的逻辑违规能量
    energy: f64,
}

impl STPContext {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            energy: 0.0,
        }
    }

    /// 执行一系列动作并计算总能量
    pub fn calculate_energy(&mut self, actions: &[ProofAction]) -> f64 {
        self.energy = 0.0; 
        self.symbol_table.clear(); 

        if actions.is_empty() {
            return 1000.0; // 惩罚空路径
        }

        for action in actions {
            match action {
                // [Fix] 匹配强类型的 value_type
                ProofAction::Define { symbol, value_type } => {
                    let val = match value_type {
                        LogicType::Even => LogicValue::Vector(vec![1.0, 0.0]),
                        LogicType::Odd => LogicValue::Vector(vec![0.0, 1.0]),
                        // 其他类型暂时映射为标量0，后续可扩展
                        _ => LogicValue::Scalar(0.0),
                    };
                    self.symbol_table.insert(symbol.clone(), val);
                }
                
                // [Fix] 处理 Transform 动作
                ProofAction::Transform { target, rule: _ } => {
                    // 简单的存在性检查
                    if !self.symbol_table.contains_key(target) {
                        self.energy += 10.0; // 引用错误惩罚
                    }
                    // TODO: 在此处实现具体的矩阵乘法 M * V
                }

                ProofAction::Assert { condition } => {
                    self.evaluate_assertion(condition);
                }
                
                ProofAction::NoOp => {
                    // NoOp 不产生能量变化
                }
            }
        }

        self.energy
    }

    /// 评估断言 (简化版)
    fn evaluate_assertion(&mut self, condition: &str) {
        // 这里的逻辑保持简单，仅演示与 STPContext 的交互
        let n_vec = self.get_vector("n");
        let m_vec = self.get_vector("m");

        // 仅处理 "(n + m) is ..." 的特例逻辑作为演示
        if condition.contains("n + m") || condition.contains("sum") {
             if let (Some(n), Some(m)) = (n_vec, m_vec) {
                let n_is_odd = n.get(1).unwrap_or(&0.0) > &0.5;
                let m_is_odd = m.get(1).unwrap_or(&0.0) > &0.5;
                
                // XOR 逻辑: 奇+奇=偶, 偶+偶=偶, 奇+偶=奇
                let sum_is_odd = n_is_odd ^ m_is_odd;

                let expected_odd = if condition.contains("Odd") {
                    true
                } else if condition.contains("Even") {
                    false
                } else {
                    self.energy += 50.0; // 未知断言类型
                    return;
                };

                if sum_is_odd != expected_odd {
                    self.energy += 100.0; // 逻辑矛盾
                }
            } else {
                // 如果断言涉及变量但变量未定义，暂不严重惩罚，因为可能是随机生成的断言
                // self.energy += 10.0; 
            }
        }
        // 其他断言忽略或给微小奖励
    }

    fn get_vector(&self, symbol: &str) -> Option<&Vec<f64>> {
        match self.symbol_table.get(symbol) {
            Some(LogicValue::Vector(v)) => Some(v),
            _ => None,
        }
    }
}
