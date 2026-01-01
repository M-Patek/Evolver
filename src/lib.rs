use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use num_bigint::{BigInt, Sign};
use num_traits::{Num, Zero};
use crate::soul::algebra::ClassGroupElement;
use crate::will::optimizer::VapoOptimizer;
use crate::body::decoder::BodyProjector;
use crate::body::adapter::SemanticAdapter;
use crate::dsl::stp_bridge::STPContext;
use crate::will::perturber::EnergyEvaluator;
use crate::dsl::schema;

pub mod soul;
pub mod will;
pub mod body;
pub mod dsl;

struct STPEvaluator;
impl EnergyEvaluator for STPEvaluator {
    fn evaluate(&self, actions: &[crate::body::adapter::ProofAction]) -> f64 {
        let mut stp = STPContext::new();
        stp.calculate_energy(actions)
    }
}

#[pyclass(name = "ProofBundle")]
#[derive(Clone)]
pub struct PyProofBundle {
    #[pyo3(get)]
    pub context_hash: String,
    
    #[pyo3(get)]
    pub discriminant_hex: String,
    
    #[pyo3(get)]
    pub start_seed_a: String,
    
    #[pyo3(get)]
    pub final_state_a: String,
    
    #[pyo3(get)]
    pub perturbation_trace: Vec<usize>,
    
    #[pyo3(get)]
    pub logic_path: Vec<String>,
    
    #[pyo3(get)]
    pub energy: f64,
}

impl From<schema::ProofBundle> for PyProofBundle {
    fn from(b: schema::ProofBundle) -> Self {
        PyProofBundle {
            context_hash: b.context_hash,
            discriminant_hex: b.discriminant_hex,
            start_seed_a: b.start_seed_a,
            final_state_a: b.final_state_a,
            perturbation_trace: b.perturbation_trace,
            logic_path: b.logic_path,
            energy: b.energy,
        }
    }
}

#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    m.add_class::<PyProofBundle>()?; 
    Ok(())
}

/// [架构升级] 
/// PyEvolver 现在是无状态的（相对于具体的代数宇宙）。
/// 它只持有 "扰动规模" (k) 等超参数配置。
/// 判别式 Δ 完全由每次 align 调用的 context 决定。
#[pyclass]
struct PyEvolver {
    k: usize,
}

#[pymethods]
impl PyEvolver {
    /// 构造函数不再需要 discriminant_hex
    /// k: 扰动集合的大小 (影响搜索宽度)
    #[new]
    fn new(k: usize) -> PyResult<Self> {
        Ok(PyEvolver { k })
    }

    /// 对齐逻辑 (Align Logic)
    /// 
    /// 现在执行严格的 "Proof of Will" 流程：
    /// 1. Context -> Hash -> Δ (Unique Universe)
    /// 2. Hash -> Seed (Unique Start)
    /// 3. Evolution -> Zero Energy
    fn align(&self, context: String) -> PyResult<PyProofBundle> {
        // 1. Inception: 动态生成宇宙和种子
        // 这里会进行素数搜索，确保 Δ 是由 context 唯一决定的
        let (seed, universe) = ClassGroupElement::spawn_universe(&context);
        
        let ctx_hash = universe.context_hash;
        let discriminant_hex = universe.discriminant.to_str_radix(16);
        let start_seed_clone = seed.clone();
        
        // 2. The Will: 在这个特定的 Δ 宇宙中进行优化
        let mut optimizer = VapoOptimizer::new(seed);
        
        let mut best_energy = f64::MAX;
        let mut best_path_digits = Vec::new();
        let mut best_actions_strings = Vec::new();
        let mut best_state = start_seed_clone.clone();
        
        let p_projection = 997; 
        let evaluator = STPEvaluator;
        let max_iterations = 50; 

        // 3. Evolution Loop
        for _ in 0..max_iterations {
            let (candidate_state, gen_idx) = optimizer.perturb();
            
            // 投影和评估
            let path_digits = BodyProjector::project(&candidate_state, self.k, p_projection);
            let logic_actions = SemanticAdapter::materialize(&path_digits);
            let energy = evaluator.evaluate(&logic_actions);

            if energy < best_energy {
                best_energy = energy;
                best_path_digits = path_digits;
                best_state = candidate_state.clone();
                best_actions_strings = logic_actions.iter().map(|a| format!("{:?}", a)).collect();
                
                optimizer.accept(candidate_state, gen_idx);
            } else {
                optimizer.reject();
            }

            if best_energy == 0.0 {
                break;
            }
        }

        // 4. Revelation
        let bundle = schema::ProofBundle {
            context_hash: ctx_hash,
            discriminant_hex: discriminant_hex, // 现在的 Δ 是动态生成的
            start_seed_a: start_seed_clone.a.to_string(),
            final_state_a: best_state.a.to_string(),
            perturbation_trace: optimizer.trace.clone(),
            logic_path: best_actions_strings,
            energy: best_energy,
        };

        Ok(PyProofBundle::from(bundle))
    }
}
