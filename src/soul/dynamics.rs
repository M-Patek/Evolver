use crate::soul::algebra::{IdealClass, Quaternion};

/// Defines how the Soul evolves over 'Time' or 'Search Steps'.
/// In the Ontological Amendment, Dynamics are no longer just "squaring matrices".
/// They are walks on the Arithmetic Lattice driven by Hecke Operators.
pub trait TimeEvolution {
    fn next(&self, state: &IdealClass) -> IdealClass;
}

/// Identity Dynamics: The state does not change implicitly.
/// Used for 'fast' mode or when the Will (Optimizer) fully controls the path.
pub struct IdentityDynamics;

impl TimeEvolution for IdentityDynamics {
    fn next(&self, state: &IdealClass) -> IdealClass {
        state.clone()
    }
}

/// Hecke Dynamics: Represents the inevitable flow of causality.
/// Instead of VDF (Squaring), we apply a deterministic Hecke Operator T_flow.
/// This simulates the "Arrow of Time" in the non-commutative space.
pub struct HeckeDynamics {
    flow_operator: Quaternion,
}

impl HeckeDynamics {
    /// Creates a new causal time flow.
    /// The 'difficulty' parameter determines the complexity of the time operator.
    pub fn new(difficulty: usize) -> Self {
        // We construct a 'Time Operator' that pushes the state forward.
        // For simplicity, we use a basic quaternion scaled by difficulty.
        // Real implementation would find a specific generator of large norm.
        let seed = difficulty as i64;
        Self {
            flow_operator: Quaternion::new(seed, 1, 0, 0),
        }
    }
}

impl TimeEvolution for HeckeDynamics {
    fn next(&self, state: &IdealClass) -> IdealClass {
        // Time is a non-commutative operator applied from the right.
        // S(t+1) = S(t) * T_flow
        state.apply_hecke(&self.flow_operator)
    }
}

/// (Deprecated) VDF Dynamics kept for backward compatibility if needed,
/// but re-implemented using Quaternion squaring (which is also non-commutative).
pub struct VDFDynamics {
    iterations: usize,
}

impl VDFDynamics {
    pub fn new(iterations: usize) -> Self {
        Self { iterations }
    }
}

impl TimeEvolution for VDFDynamics {
    fn next(&self, state: &IdealClass) -> IdealClass {
        let mut current_q = state.value;
        // Repeatedly square the quaternion to simulate VDF delay
        for _ in 0..self.iterations {
            current_q = current_q * current_q;
        }
        
        IdealClass {
            value: current_q,
            discriminator: state.discriminator,
        }
    }
}
