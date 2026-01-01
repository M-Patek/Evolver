// src/dsl/schema.rs
//
// 喵呜！这是 Evolver DSL 的核心架构定义文件 (Refined Version)。
// [Hash Strategy Alignment]: 
// 所有的哈希字段统一升级为 String 类型，以承载 SHA-256 Hex。
// 引入 ProofBundle 作为标准产物定义。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 意志证明包 (Proof of Will Bundle) - 标准定义
/// 
/// [Alignment Fix]:
/// 之前 ProofBundle 未接入主流程且哈希格式不一致。
/// 现在这是系统唯一的真理凭证结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofBundle {
    /// 上下文的 SHA-256 哈希 (Hex String)
    /// 替代了旧版的 context_id (u64)
    pub context_hash: String,
    
    /// 使用的判别式 (Hex String)
    pub discriminant_hex: String,
    
    /// 初始种子的 'a' 系数 (String)
    pub start_seed_a: String,
    
    /// 最终真理状态的 'a' 系数 (String)
    pub final_state_a: String,
    
    /// 扰动轨迹 (Trace)
    /// 记录了优化器在凯莱图上走的每一步 (Generator Index 序列)
    pub perturbation_trace: Vec<usize>,
    
    /// 最终生成的逻辑路径 (人类可读)
    pub logic_path: Vec<String>,
    
    /// 最终能量 (应为 0.0)
    pub energy: f64,
}

/// 模型的顶层定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionModel {
    pub name: String,
    pub spaces: HashMap<String, SpaceDef>,
    pub state_vars: HashMap<String, VariableDef>,
    pub control_vars: HashMap<String, VariableDef>,
    pub parameters: HashMap<String, ParameterDef>,
    pub dynamics: Vec<DynamicEquation>,
    pub constraints: Vec<Constraint>,
    pub objective: Option<Objective>,
    pub perturbation: Option<PerturbationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpaceDef {
    pub name: String,
    pub type_: SpaceType,
    pub dim: usize, 
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpaceType {
    Euclidean,      
    LieGroup(String), 
    Manifold(String), 
    Boolean,        
    Custom(String), 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDef {
    pub name: String,
    pub space: String, 
    pub shape: Vec<usize>,
    pub initial_value: Option<Expression>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    pub value: ParameterValue,
    pub learnable: bool, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Scalar(f64),
    Vector(Vec<f64>),
    Matrix(Vec<Vec<f64>>), 
    Tensor(Vec<usize>, Vec<f64>), 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicEquation {
    pub type_: DynamicType,
    pub lhs: Expression, 
    pub rhs: Expression,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DynamicType {
    Continuous, 
    Discrete,   
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Var(String),
    Param(String),
    Constant(f64),
    VectorLiteral(Vec<f64>),
    MatrixLiteral(Vec<Vec<f64>>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>), 
    Div(Box<Expression>, Box<Expression>),
    SemiTensorProd(Box<Expression>, Box<Expression>),
    KroneckerProd(Box<Expression>, Box<Expression>),
    LieBracket(Box<Expression>, Box<Expression>),
    FunctionCall {
        func_name: String,
        args: Vec<Expression>,
    },
    Derivative {
        target: Box<Expression>,
        wrt: String, 
        order: u8,
    },
    NextState(Box<Expression>),
    Transpose(Box<Expression>),
    Norm(Box<Expression>, String), 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub expr: Expression,
    pub kind: ConstraintKind,
    pub type_: ConstraintType, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintKind {
    Equality,   
    Inequality, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Hard,
    Soft { weight: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub kind: ObjectiveKind,
    pub expr: Expression, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveKind {
    Minimize,
    Maximize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerturbationConfig {
    pub method: PerturbationMethod,
    pub global_decay: f64,
    pub targets: Vec<String>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerturbationMethod {
    Gaussian { 
        mean: f64, 
        std_dev: f64 
    },
    Langevin { 
        temperature: f64, 
        friction_coeff: f64 
    },
    ValuationAdaptive {
        base_intensity: f64,
        adaptation_rate: f64, 
        valuation_sensitivity: f64, 
    },
}

impl Expression {
    pub fn stp(lhs: Expression, rhs: Expression) -> Self {
        Expression::SemiTensorProd(Box::new(lhs), Box::new(rhs))
    }

    pub fn add(lhs: Expression, rhs: Expression) -> Self {
        Expression::Add(Box::new(lhs), Box::new(rhs))
    }
    
    pub fn mul(lhs: Expression, rhs: Expression) -> Self {
        Expression::Mul(Box::new(lhs), Box::new(rhs))
    }
}
