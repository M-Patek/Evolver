use crate::soul::algebra::Quaternion;

/// 追踪器 (Tracer)
/// 负责记录“意志”的决策路径。
/// 在本体论重构后，Trace 不再是状态的快照，而是 **因果算子 (Operators)** 的序列。
/// 
/// 验证逻辑： FinalState = InitialState * q1 * q2 * ... * qn
#[derive(Debug, Clone)]
pub struct Trace {
    /// 记录每一步施加的 Hecke 算子
    pub path: Vec<Quaternion>,
    /// 记录路径的总能量变化（用于分析收敛性）
    pub energy_log: Vec<f64>,
}

impl Trace {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            energy_log: Vec::new(),
        }
    }

    /// 记录一步移动
    pub fn record(&mut self, operator: Quaternion, energy: f64) {
        self.path.push(operator);
        self.energy_log.push(energy);
    }

    /// 获取路径长度（即因果链的深度）
    pub fn len(&self) -> usize {
        self.path.len()
    }

    /// 导出为简单的算子列表，用于 Proof Bundle
    pub fn to_proof_sequence(&self) -> Vec<Quaternion> {
        self.path.clone()
    }
}
