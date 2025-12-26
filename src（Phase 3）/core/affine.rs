// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::algebra::ClassGroupElement;
use rug::Integer;

/// ⚠️ [Safety Limit]: 局部算子 P 因子最大位宽
/// 限制为 8192 bits。这足以聚合 ~128 个 Token (假设每个 Token 64 bits)，
/// 但严禁用于全局历史累积。这从根本上杜绝了 P 因子爆炸问题。
const MAX_CHUNK_P_BITS: u32 = 8192;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
    /// 
    /// 用于时间线上的因果累积。
    /// 公式: (P1, Q1) ⊕ (P2, Q2) = (P1*P2, Q1^P2 * Q2)
    /// 这里的 Q1^P2 引入了非交换性，确保历史顺序不可篡改。
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // [SAFETY CHECK]: 防止 P 因子爆炸
        let p_bits_new = self.p_factor.significant_bits() + other.p_factor.significant_bits();
        if p_bits_new > MAX_CHUNK_P_BITS { 
             return Err(format!(
                 "⛔ Security Halt: Affine P-Factor overflow ({} bits). \
                 Global accumulation is forbidden. Use `apply_affine` for state evolution.", 
                 p_bits_new
             ));
        }

        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Composition Law: (P1, Q1) + (P2, Q2) = (P1*P2, Q1^P2 * Q2)
        // 注意顺序：先应用 other 的 P2 到 self 的 Q1，再加上 other 的 Q2
        // 这里体现了非交换性：S ^ (P1*P2) * (Q1^P2 * Q2)
        let q1_pow_p2 = self.q_shift.pow(&other.p_factor, discriminant)?;
        let new_q = q1_pow_p2.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    /// 🌌 [Space Operator]: Commutative Aggregation (空间聚合 - 交换)
    /// 
    /// 理论修正 (Theoretical Fix):
    /// 为了保证多维全息验证的数学正确性 (Fold_xy == Fold_yx)，
    /// 空间维度的聚合必须是交换的 (Abelian)。
    /// 我们利用 Class Group 本身是阿贝尔群的性质，执行分量乘法。
    /// 
    /// 公式: (P1, Q1) ⊗ (P2, Q2) = (P1*P2, Q1*Q2)
    pub fn commutative_merge(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // P_new = P1 * P2 (整数乘法，交换)
        // P 因子依然用于位置指纹验证
        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Q_new = Q1 * Q2 (群乘法，交换)
        // [CRITICAL CHANGE]: 移除了 Q^P 的非交换操作
        // 这使得 Fold 操作在拓扑上变得可交换。
        let new_q = self.q_shift.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    /// 逆向操作辅助函数：用于 Oracle 提取
    pub fn try_divide_p(&self, denominator: &Integer) -> Option<Integer> {
        let (quotient, rem) = self.p_factor.div_rem_ref(denominator).into();
        if rem == Integer::from(0) {
            Some(quotient)
        } else {
            None
        }
    }
}
