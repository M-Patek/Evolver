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

#[pyclass]
pub struct PyEvolver {
    p: u64, 
    k: u64, 
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
            search_steps: search_steps.unwrap_or(100),
        }
    }

    #[pyo3(signature = (context, mode="prove", depth=16))]
    pub fn align(&self, context: String, mode: &str, depth: usize) -> PyResult<Vec<u64>> {
        let seed = IdealClass::from_hash(&context, self.p); 
        let eval_projector = Projector::new(self.p);

        let (dynamics, evaluator): (Box<dyn TimeEvolution>, Box<dyn Evaluator>) = match mode {
            "fast" | "native" => (
                Box::new(IdentityDynamics),   
                Box::new(GeometricEvaluator), 
            ),
            "prove" | "vdf" => (
                Box::new(VDFDynamics::new(self.vdf_difficulty)), 
                // Evaluator 内部现在使用 Split Projection 逻辑
                Box::new(StpEvaluator::new(eval_projector, depth)), 
            ),
            "hybrid" => (
                Box::new(VDFDynamics::new(self.vdf_difficulty)), 
                Box::new(GeometricEvaluator),                    
            ),
            _ => return Err(pyo3::exceptions::PyValueError::new_err(
                "Unknown mode. Available modes: 'fast', 'prove', 'hybrid'",
            )),
        };

        // The Will searches...
        let optimizer = VapoOptimizer::new(evaluator, self.search_steps);
        let optimized_state = optimizer.search(&seed);

        // The Body manifests...
        let mut logic_path = Vec::with_capacity(depth);
        let mut state = optimized_state;
        
        // [CRITICAL] Output Generation must use EXACT projection
        let out_projector = Projector::new(self.p);

        for t in 0..depth {
            // 使用 project_exact 确保输出的逻辑是代数状态的唯一指纹
            let digit = out_projector.project_exact(&state, t as u64);
            logic_path.push(digit);

            state = dynamics.next(&state);
        }

        Ok(logic_path)
    }
}

#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    Ok(())
}
