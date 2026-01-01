// src/dsl/schema.rs
//
// 喵呜！这是 Evolver DSL 的核心架构定义文件 (Refined Version)。
// 它定义了语言的抽象语法树 (AST) 和语义结构，
// 负责将解析器 (Parser) 的结果传递给数学内核 (Math Kernel)。
//
// 深度重构与优化记录 (Deep Optimization Log):
// 1. [Algebra] 引入 `shape` (张量形状) 到 `VariableDef`，为了精准适配 src/soul/algebra.rs 中的 STP 矩阵维度检查。
// 2. [Will] 重构 `PerturbationMethod` 为带数据的枚举，直接承载 v-PuNNs 和 Langevin 动力学的超参数。
// 3. [Theory] 区分 `ConstraintType` (Hard/Soft)，对应 Constraint_Semantics.md 中的惩罚函数逻辑。
// 4. [Dynamics] 明确 `DynamicType`，区分离散映射 (Map) 和连续流 (Flow)，适配 Lipschitz 论文中的不同系统描述。
// 5. [State] 增加 `initial_value`，闭环演化系统的定义。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 模型的顶层定义，代表一个完整的 Evolver 问题描述。
/// 对应 IDEAL_MODEL_SPEC.md 中的结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionModel {
    /// 模型名称
    pub name: String,
    
    /// 拓扑空间定义（流形、向量空间等）
    /// 对应 src/body/topology.rs
    pub spaces: HashMap<String, SpaceDef>,
    
    /// 状态变量定义 (State Variables)
    pub state_vars: HashMap<String, VariableDef>,
    
    /// 控制/输入变量定义 (Control Inputs)
    pub control_vars: HashMap<String, VariableDef>,
    
    /// 参数定义 (Parameters)
    pub parameters: HashMap<String, ParameterDef>,
    
    /// 动力学方程或演化规则 (Evolution Rules / Dynamics)
    /// 例如: dx/dt = f(x, u) 或 x_{k+1} = M \ltimes x_k
    pub dynamics: Vec<DynamicEquation>,
    
    /// 约束条件 (Constraints)
    /// 对应 theory/Constraint_Semantics.md
    pub constraints: Vec<Constraint>,
    
    /// 优化目标 (Objective / Energy)
    /// 对应 theory/Unified_Energy_Metric.md
    pub objective: Option<Objective>,
    
    /// 摄动配置 (Perturbation Configuration)
    /// 对应 src/will/perturber.rs
    pub perturbation: Option<PerturbationConfig>,
}

/// 空间定义：定义变量存在的数学空间
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpaceDef {
    pub name: String,
    pub type_: SpaceType,
    /// 拓扑维度 (Topological Dimension)
    /// 注意：对于流形，这是其内在维度；对于矩阵空间，这是自由度。
    /// 具体的张量形状由 VariableDef 中的 shape 定义。
    pub dim: usize, 
}

/// 空间类型枚举
/// 必须与 src/body/topology.rs 中的逻辑兼容
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpaceType {
    Euclidean,      // 欧几里得空间 R^n
    LieGroup(String), // 李群 (如 SO(3), SE(3))
    Manifold(String), // 一般流形 (Riemannian, Symplectic etc.)
    Boolean,        // 布尔网络空间 (用于逻辑矩阵STP)
    Custom(String), // 自定义空间
}

/// 变量定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDef {
    pub name: String,
    pub space: String, // 关联到 spaces 中的 key
    
    /// 张量形状 [rows, cols, depth, ...]
    /// 对于 STP (半张量积) 运算，必须明确知道矩阵的行列数。
    /// 例如：一个 2x2 的旋转矩阵 shape 为 [2, 2]。
    /// 对于普通向量 R^n，shape 为 [n]。
    pub shape: Vec<usize>,
    
    /// 初始值 (Initial Condition)
    /// 动力学演化的起点 x_0
    pub initial_value: Option<Expression>,
    
    pub description: Option<String>,
}

/// 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    pub value: ParameterValue,
    /// 是否在优化过程中可变 (Learnable parameters)
    pub learnable: bool, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Scalar(f64),
    Vector(Vec<f64>),
    Matrix(Vec<Vec<f64>>), // 行主序
    Tensor(Vec<usize>, Vec<f64>), // Shape + Flat Data
}

/// 动力学方程: LHS = RHS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicEquation {
    /// 动力学类型：连续 (ODE) 或 离散 (Map)
    pub type_: DynamicType,
    pub lhs: Expression, // 通常是导数 d(var)/dt 或 下一时刻 var'
    pub rhs: Expression,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DynamicType {
    Continuous, // Differential Equation (Flow), e.g., dx/dt = ...
    Discrete,   // Difference Equation (Map), e.g., x[k+1] = ...
}

/// 数学表达式枚举
/// 这是 DSL 的核心，用于构建计算图。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    /// 变量引用
    Var(String),
    /// 参数引用
    Param(String),
    /// 常数
    Constant(f64),
    /// 向量字面量
    VectorLiteral(Vec<f64>),
    /// 矩阵字面量 (行主序)
    MatrixLiteral(Vec<Vec<f64>>),
    
    /// 基本算术运算
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    /// 普通矩阵乘法或标量乘法 (Standard Matrix Multiplication)
    Mul(Box<Expression>, Box<Expression>), 
    Div(Box<Expression>, Box<Expression>),
    
    /// 半张量积 (Semi-Tensor Product)
    /// 对应 src/soul/algebra.rs 和 src/dsl/stp_bridge.rs
    /// 符号通常是 \ltimes，不仅支持同维矩阵，还支持任意维度匹配。
    SemiTensorProd(Box<Expression>, Box<Expression>),
    
    /// 克罗内克积 (Kronecker Product)
    KroneckerProd(Box<Expression>, Box<Expression>),
    
    /// 李括号 (Lie Bracket) [A, B] = AB - BA
    /// 对应李代数结构
    LieBracket(Box<Expression>, Box<Expression>),
    
    /// 函数调用 (如 sin, cos, exp, sigmoid, tanh 或自定义映射)
    FunctionCall {
        func_name: String,
        args: Vec<Expression>,
    },
    
    /// 微分算子 (d/dt, partial/partial_x)
    Derivative {
        target: Box<Expression>,
        wrt: String, // with respect to variable name
        order: u8,
    },
    
    /// 状态算子：下一时刻 (用于离散系统) x[k+1] -> Next(x)
    NextState(Box<Expression>),
    
    /// 转置
    Transpose(Box<Expression>),
    
    /// 范数
    Norm(Box<Expression>, String), // 第二个参数是范数类型 ("L2", "L1", "Frobenius")
}

/// 约束类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub expr: Expression,
    pub kind: ConstraintKind,
    /// 约束性质：硬约束必须满足，软约束作为惩罚项
    pub type_: ConstraintType, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintKind {
    Equality,   // expr == 0
    Inequality, // expr <= 0 (标准形式)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Hard,
    /// Soft 约束包含权重，用于构造罚函数 (Penalty Function)
    Soft { weight: f64 },
}

/// 优化目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub kind: ObjectiveKind,
    pub expr: Expression, // 定义能量函数 E(x)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveKind {
    Minimize,
    Maximize,
}

/// 摄动配置
/// 用于指导 src/will/perturber.rs 如何进行随机游走或扰动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerturbationConfig {
    pub method: PerturbationMethod,
    /// 全局衰减因子，用于随着时间减少摄动幅度 (Simulated Annealing concepts)
    pub global_decay: f64,
    pub targets: Vec<String>, // 应用于哪些变量
}

/// 摄动方法枚举
/// 详细对应各种随机过程和 v-PuNNs 策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerturbationMethod {
    /// 高斯白噪声
    Gaussian { 
        mean: f64, 
        std_dev: f64 
    },
    
    /// 郎之万动力学 (Langevin Dynamics)
    /// 引入梯度信息和布朗运动
    Langevin { 
        temperature: f64, 
        friction_coeff: f64 
    },
    
    /// 估值自适应摄动 (Valuation-Adaptive)
    /// 对应 "v-PuNNs" 论文，根据当前能量/估值动态调整扰动
    ValuationAdaptive {
        base_intensity: f64,
        adaptation_rate: f64, // alpha
        valuation_sensitivity: f64, // beta
    },
}

// -------------------------------------------
// 辅助构造函数 (Helper Constructors)
// -------------------------------------------

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
