// src/dsl/schema.rs
// 单一真理源 (SSOT): 定义系统中通用的数据结构和逻辑类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==========================================
// [New] 核心逻辑类型定义 (修复类型流离失所)
// ==========================================

/// 逻辑类型枚举
/// 提升为全局枚举，确保 Adapter 和 STP Bridge 对类型的理解一致
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogicType {
    Even,
    Odd,
    Prime,
    Integer,
    Unknown, // 用于处理熵不足或解析失败的情况
}

/// 统一的证明动作 (Proof Action)
/// 融合了 Adapter 的生成能力和 Bridge 的执行需求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProofAction {
    /// 定义语义: symbol 绑定到 specific LogicType
    Define { 
        symbol: String, 
        value_type: LogicType // 使用强类型，而非 String
    },
    
    /// 变换语义: 对 target 应用 rule
    Transform { 
        target: String, 
        rule: String 
    },
    
    /// 断言语义: 检查条件
    Assert { 
        condition: String 
    },
    
    /// 空操作
    NoOp,
}

/// 逻辑值 (用于 STP 计算引擎)
#[derive(Debug, Clone, PartialEq)]
pub enum LogicValue {
    Scalar(f64),
    Vector(Vec<f64>), // 对应 STP 的向量表示
}

// ==========================================
// 原有的 ProofBundle 和 Model 定义 (保持兼容)
// ==========================================

/// 生成元规格说明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorSpec {
    pub algorithm_version: String,
    pub count: usize,
    pub max_norm: Option<u64>,
}

/// 意志证明包 (Proof of Will Bundle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofBundle {
    pub context_hash: String,
    pub discriminant_hex: String,
    pub start_seed_a: String,
    pub final_state_a: String,
    pub generator_spec: GeneratorSpec,
    pub perturbation_trace: Vec<usize>,
    pub logic_path: Vec<String>,
    pub energy: f64,
}

// (以下是 EvolutionModel 相关定义，保持原样以支持完整性)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionModel {
    pub name: String,
    pub spaces: HashMap<String, SpaceDef>,
    // ... 其他字段可根据需求扩展，这里保留最简结构以通过编译
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpaceDef {
    pub name: String,
    pub type_: String,
    pub dim: usize, 
}
