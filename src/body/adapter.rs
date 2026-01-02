/// 适配器 (Adapter)
/// 负责将 Projector 产生的原始 u64 熵转化为具体的领域逻辑。
///
/// 在本体论修正案中，这代表了从“四元数格”到“逻辑电路”的物质化过程。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOp {
    And,
    Or,
    Xor,
    Not,
    Identity,
}

#[derive(Debug, Clone)]
pub struct ProofAction {
    pub op: LogicOp,
    pub operand_idx_1: usize,
    pub operand_idx_2: usize,
    pub output_idx: usize,
}

pub struct Adapter;

impl Adapter {
    /// 将代数投影值转化为逻辑动作
    /// discrete_entropy: 来自 Projector::project_exact 的输出
    /// memory_size: 当前逻辑系统的可用寄存器数量
    pub fn materialize(discrete_entropy: u64, memory_size: usize) -> ProofAction {
        // 使用位掩码和移位来确定性地解析动作
        // 这种做法类似于从基因 (DNA) 表达为蛋白质
        
        // Bit 0-7: OpCode
        let op_code = discrete_entropy & 0xFF;
        
        // Bit 8-23: Operand 1
        let op1 = ((discrete_entropy >> 8) & 0xFFFF) as usize % memory_size;
        
        // Bit 24-39: Operand 2
        let op2 = ((discrete_entropy >> 24) & 0xFFFF) as usize % memory_size;
        
        // Bit 40-55: Output
        let out = ((discrete_entropy >> 40) & 0xFFFF) as usize % memory_size;

        let op = match op_code % 5 {
            0 => LogicOp::And,
            1 => LogicOp::Or,
            2 => LogicOp::Xor,
            3 => LogicOp::Not,
            _ => LogicOp::Identity,
        };

        ProofAction {
            op,
            operand_idx_1: op1,
            operand_idx_2: op2,
            output_idx: out,
        }
    }
}
