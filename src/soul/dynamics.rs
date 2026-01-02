use crate::soul::algebra::IdealClass;

/// TimeEvolution defines how an algebraic state evolves over the temporal dimension.
///
/// This trait decouples the "Proof-of-Will" (VDF) mechanism from the core state,
/// allowing the system to run in "Fast Mode" (Identity) or "Rigorous Mode" (VDF).
pub trait TimeEvolution {
    /// Evolve the current state to the next time step.
    ///
    /// # Arguments
    /// * `current` - The state at time t.
    ///
    /// # Returns
    /// The state at time t+1.
    fn next(&self, current: &IdealClass) -> IdealClass;

    /// Returns a descriptor of the dynamics (for logging/verification).
    fn name(&self) -> &'static str;
}

/// Identity Dynamics (Fast Mode).
///
/// In this mode, the algebraic state does not undergo complex squaring operations
/// between logical steps. The variation in logic comes solely from the
/// Linear Congruence Projection's index `k` (i.e., `a + k*b`).
///
/// Cost: O(1) Copy.
/// Use case: Real-time generation, Code completion.
pub struct IdentityDynamics;

impl TimeEvolution for IdentityDynamics {
    fn next(&self, current: &IdealClass) -> IdealClass {
        // The truth remains constant; only the perspective (projection index) changes.
        current.clone()
    }

    fn name(&self) -> &'static str {
        "Identity (Fast)"
    }
}

/// VDF Dynamics (Proof-of-Will Mode).
///
/// Enforces sequential computation via repeated squaring in the Class Group.
/// This acts as a Verifiable Delay Function (VDF).
///
/// Cost: O(difficulty) Group Operations.
/// Use case: Proof of Will, High-value logic verification.
pub struct VDFDynamics {
    /// Number of squarings per time step.
    pub difficulty: usize,
}

impl VDFDynamics {
    pub fn new(difficulty: usize) -> Self {
        Self { difficulty }
    }
}

impl TimeEvolution for VDFDynamics {
    fn next(&self, current: &IdealClass) -> IdealClass {
        let mut state = current.clone();
        // Force sequential work
        for _ in 0..self.difficulty {
            state = state.square();
        }
        state
    }

    fn name(&self) -> &'static str {
        "VDF (Proof-of-Will)"
    }
}
