use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use num_bigint::BigInt;

use crate::soul::algebra::ClassGroupElement;
use crate::body::topology::VPuNNConfig;
use crate::dsl::stp_bridge::STPContext;
use crate::dsl::schema::ProofAction;
use crate::will::perturber::EnergyEvaluator;
use crate::will::optimizer;
use crate::body::decoder; // [Added] 引入解码器

pub mod dsl;
pub mod soul;
pub mod body {
    pub mod topology;
    pub mod projection;
    pub mod decoder;
    pub mod adapter;
}
pub mod will {
    pub mod optimizer;
    pub mod perturber;
}

// ... StpBridge 实现保持不变 ...
struct StpBridge<'a> {
    context: &'a RefCell<STPContext>,
}

impl<'a> EnergyEvaluator for StpBridge<'a> {
    fn evaluate(&self, path: &[u64]) -> f64 {
        // [Logic Decoding]
        let decision_seed = path.get(0).unwrap_or(&0);
        
        let action = if decision_seed % 2 == 0 {
            ProofAction::Define {
                symbol: "sum_truth".to_string(),
                hierarchy_path: vec!["Even".to_string()]
            }
        } else {
            ProofAction::Define {
                symbol: "sum_truth".to_string(),
                hierarchy_path: vec!["Odd".to_string()]
            }
        };

        let mut stp = self.context.borrow_mut();
        stp.calculate_energy(&action);

        let check_action = ProofAction::Apply {
            theorem_id: "ModAdd".to_string(),
            inputs: vec!["n".to_string(), "m".to_string()],
            output_symbol: "sum_truth".to_string(),
        };

        stp.calculate_energy(&check_action)
    }
}

#[pyclass]
pub struct PyEvolver {
    soul: ClassGroupElement, 
    body: VPuNNConfig,
    stp: RefCell<STPContext>, 
}

#[pymethods]
impl PyEvolver {
    #[new]
    fn new(p: u64, k: usize) -> Self {
        println!("🐱 PyEvolver Initializing with p={}, k={}...", p, k);

        let mut stp_ctx = STPContext::new();
        let setup_n = ProofAction::Define { 
            symbol: "n".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        };
        let setup_m = ProofAction::Define { 
            symbol: "m".to_string(), 
            hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
        };
        stp_ctx.calculate_energy(&setup_n);
        stp_ctx.calculate_energy(&setup_m);

        let discriminant = BigInt::from(-23);
        let identity_soul = ClassGroupElement::identity(&discriminant);
        let body_config = VPuNNConfig::new(k, p);

        PyEvolver {
            soul: identity_soul,
            body: body_config,
            stp: RefCell::new(stp_ctx),
        }
    }

    fn align(&mut self, context: String) -> Vec<u64> {
        // 1. 种子注入 (Context Seeding)
        let mut hasher = DefaultHasher::new();
        context.hash(&mut hasher);
        let seed = hasher.finish();
        
        // 演化灵魂
        self.soul = self.soul.evolve(seed);

        // 2. 优化 (Optimization)
        let evaluator = StpBridge { context: &self.stp };
        
        // [Architecture Fix]: 将 body config 传入优化器
        // 确保优化器使用的是正确的投影几何
        let optimized_soul = optimizer::optimize(&self.soul, &self.body, &evaluator);

        self.soul = optimized_soul;
        
        // 3. 物质化 (Materialization)
        // [Architecture Fix]: 使用标准的 decoder 生成最终路径
        // 不再使用本地闭包，确保 Python 拿到的结果与优化器看到的一致
        decoder::materialize_path(&self.soul, &self.body)
    }
}

#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    Ok(())
}
