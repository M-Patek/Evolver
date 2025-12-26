// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::algebra::ClassGroupElement;
use rug::Integer;

/// ⚠️ [Safety Limit]: 局部算子 P 因子最大位宽
/// 边界定义: 仿射因子溢出 (P-Factor Overflow)
/// 证伪意义: 防止算子无限膨胀，阻断 CPU DoS 攻击。
///
/// [Theory]: 
/// HTP 协议禁止将无限的历史压缩进单个 AffineTuple 的 P 因子中。
/// 全局演化必须使用流式处理 (Streaming)，而 P 因子累积仅限于局部 Chunk。
const MAX_CHUNK_P_BITS: u32 = 4096;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AffineTuple {
    pub p_factor: Integer,      
    pub q_shift: ClassGroupElement, 
}

impl AffineTuple {
    pub fn identity(discriminant: &Integer) -> Self {
        AffineTuple {
            p_factor: Integer::from(1),
            q_shift: ClassGroupElement::identity(discriminant),
        }
    }

    /// ⏳ [Time Operator]: Non-Commutative Composition (时间演化 - 非交换)
    /// 公式: (P1, Q1) ⊕ (P2, Q2) = (P1*P2, Q1^P2 * Q2)
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // [FALSIFIABILITY CHECK]: P-Factor Overflow (P 因子溢出熔断)
        // 这是 HTP 协议的物理边界：
        // 如果算子规模超过安全阈值 (4096 bits)，视为非法操作或 DoS 攻击，立即熔断。
        let p_bits_new = self.p_factor.significant_bits() + other.p_factor.significant_bits();
        if p_bits_new > MAX_CHUNK_P_BITS { 
             return Err(format!("❌ Falsified: Affine P-Factor overflow ({} bits > {}). Global accumulation is forbidden; use State Streaming instead.", p_bits_new, MAX_CHUNK_P_BITS));
        }

        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Composition Law: Q_new = Q1^P2 * Q2
        // 这里的 Q1^P2 引入了非交换性，任何对 P2 顺序的篡改都会导致 Q_new 剧烈变化
        let q1_pow_p2 = self.q_shift.pow(&other.p_factor, discriminant)?;
        let new_q = q1_pow_p2.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    /// 🌌 [Space Operator]: Commutative Aggregation (空间聚合 - 交换)
    /// 公式: (P1, Q1) ⊗ (P2, Q2) = (P1*P2, Q1*Q2)
    pub fn commutative_merge(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // P_new = P1 * P2 (整数乘法，交换)
        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Q_new = Q1 * Q2 (群乘法，交换)
        // 注意：这里使用的是 compose 而非 pow，确保操作是阿贝尔的
        let new_q = self.q_shift.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }
}
