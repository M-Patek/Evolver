use pyo3::prelude::*;
use crate::soul::algebra::IdealClass;
use crate::soul::dynamics::{TimeEvolution, IdentityDynamics, VDFDynamics};
use crate::will::optimizer::VapoOptimizer;
use crate::will::evaluator::{Evaluator, GeometricEvaluator, StpEvaluator};
use crate::body::projection::Projector;

pub mod soul;
pub mod will;
pub mod body;
pub mod dsl;

/// The Python Interface for Evolver.
///
/// This acts as the facade, orchestrating the interaction between
/// the Soul (Algebra), the Will (Optimizer), and the Body (Projector).
#[pyclass]
pub struct PyEvolver {
    // System Parameters
    p: u64,
    k: u64,
    
    // Configuration
    vdf_difficulty: usize,
    search_steps: usize,
}

#[pymethods]
impl PyEvolver {
    #[new]
    #[pyo3(signature = (p, k, vdf_difficulty=None, search_steps=None))]
    pub fn new(p: u64, k: u64, vdf_difficulty: Option<usize>, search_steps: Option<usize>) -> Self {
        PyEvolver {
            p,
            k,
            vdf_difficulty: vdf_difficulty.unwrap_or(1),
            search_steps: search_steps.unwrap_or(100), // Default to a reasonable search depth
        }
    }

    /// Align logic with the given context.
    ///
    /// # Arguments
    /// * `context` - The input string prompt (seed context).
    /// * `mode` - Operating mode:
    ///     - "fast" / "native": Identity dynamics + Geometric heuristic. Fast, good for code.
    ///     - "prove" / "vdf": VDF dynamics + STP rigor. Slow, produces Proof-of-Will.
    ///     - "hybrid": VDF dynamics + Geometric heuristic. Verifiable trace, but fast search.
    /// * `depth` - Length of the logical path to generate.
    #[pyo3(signature = (context, mode="prove", depth=16))]
    pub fn align(&self, context: String, mode: &str, depth: usize) -> PyResult<Vec<u64>> {
        // 1. Initialization (The Soul)
        // Instantiate the algebraic seed from the context hash.
        // (Assuming IdealClass has a constructor from hash/context)
        let seed = IdealClass::from_hash(&context, self.p); 

        // 2. Strategy Selection (The Decoupling)
        // We select the "Time Strategy" (how time flows) and the "Will Strategy" (what is good).
        let (dynamics, evaluator): (Box<dyn TimeEvolution>, Box<dyn Evaluator>) = match mode {
            "fast" | "native" => (
                Box::new(IdentityDynamics),   // Time: Instant (No VDF)
                Box::new(GeometricEvaluator), // Will: Intuitive (Geometric Form)
            ),
            "prove" | "vdf" => (
                Box::new(VDFDynamics::new(self.vdf_difficulty)), // Time: Heavy (VDF)
                Box::new(StpEvaluator),                          // Will: Rigorous (STP)
            ),
            "hybrid" => (
                Box::new(VDFDynamics::new(self.vdf_difficulty)), // Time: Heavy (VDF for security)
                Box::new(GeometricEvaluator),                    // Will: Intuitive (Fast Search)
            ),
            _ => return Err(pyo3::exceptions::PyValueError::new_err(
                "Unknown mode. Available modes: 'fast', 'prove', 'hybrid'",
            )),
        };

        // 3. The Will (Optimization)
        // The optimizer is now completely agnostic. It just minimizes the energy
        // defined by the chosen evaluator.
        let optimizer = VapoOptimizer::new(evaluator, self.search_steps);
        let optimized_state = optimizer.search(&seed);

        // 4. Materialization (The Body)
        // We use the optimized seed to spin out the universe (logic path).
        let mut logic_path = Vec::with_capacity(depth);
        let mut state = optimized_state;
        let projector = Projector::new(self.p);

        for t in 0..depth {
            // Project the current algebraic state into the logical digit
            let digit = projector.project(&state, t as u64);
            logic_path.push(digit);

            // Evolve the state using the chosen dynamics
            state = dynamics.next(&state);
        }

        Ok(logic_path)
    }
}

/// A module to wrap the Rust code for Python
#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    Ok(())
}
