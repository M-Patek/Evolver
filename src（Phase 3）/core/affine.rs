// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use super::algebra::ClassGroupElement;
use rug::Integer;

/// ⚠️ [Safety Limit]: 局部算子 P 因子最大位宽
const MAX_CHUNK_P_BITS: u32 = 8192;

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
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // [SAFETY CHECK]: 防止 P 因子爆炸
        let p_bits_new = self.p_factor.significant_bits() + other.p_factor.significant_bits();
        if p_bits_new > MAX_CHUNK_P_BITS { 
             return Err(format!("Affine P-Factor overflow ({} bits).", p_bits_new));
        }

        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Composition Law: (P1, Q1) + (P2, Q2) = (P1*P2, Q1^P2 * Q2)
        // 这里的 Q1^P2 引入了非交换性
        let q1_pow_p2 = self.q_shift.pow(&other.p_factor, discriminant)?;
        let new_q = q1_pow_p2.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    /// 🌌 [Space Operator]: Commutative Aggregation (空间聚合 - 交换)
    /// 
    /// 理论修正: (P1, Q1) ⊗ (P2, Q2) = (P1*P2, Q1*Q2)
    /// 利用 Class Group 本身是阿贝尔群的性质，移除幂运算，确保交换律。
    pub fn commutative_merge(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        // P_new = P1 * P2 (整数乘法，交换)
        let new_p = Integer::from(&self.p_factor * &other.p_factor);

        // Q_new = Q1 * Q2 (群乘法，交换)
        let new_q = self.q_shift.compose(&other.q_shift, discriminant)?;

        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }
}

// 🛡️ [Guard]: 永久性的算子性质测试
#[cfg(test)]
mod strict_tests {
    use super::*;
    use crate::phase3::core::algebra::ClassGroupElement;

    #[test]
    fn test_commutative_merge_is_abelian() {
        // Setup environment (Mock discriminant)
        let d = Integer::from(-1000003); 
        
        // Construct two distinct tuples
        // A: (P=3, Q=Generator)
        let g = ClassGroupElement::generator(&d);
        let a = AffineTuple { p_factor: Integer::from(3), q_shift: g.clone() };
        
        // B: (P=5, Q=Generator^2)
        let g2 = g.square(&d).unwrap();
        let b = AffineTuple { p_factor: Integer::from(5), q_shift: g2 };

        // 1. Calculate A ⊗ B
        let ab = a.commutative_merge(&b, &d).expect("Merge failed");

        // 2. Calculate B ⊗ A
        let ba = b.commutative_merge(&a, &d).expect("Merge failed");

        // 3. Assert Equality
        assert_eq!(ab.p_factor, ba.p_factor, "P-factors must commute");
        assert_eq!(ab.q_shift, ba.q_shift, "Q-shifts must commute (Abelian Group Violation)");
        
        // 4. Contrast with Compose (Should NOT commute)
        // 这是对照组，证明 A⊕B != B⊕A
        let ab_comp = a.compose(&b, &d).unwrap();
        let ba_comp = b.compose(&a, &d).unwrap();
        assert_ne!(ab_comp.q_shift, ba_comp.q_shift, "Time Operator SHOULD be non-commutative!");
    }
}
