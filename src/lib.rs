// src/lib.rs
// 这是 Rust 和 Python 的“外交公署”
// 我们在这里把 Rust 的强类型逻辑转换成 Python 喜欢的形式。

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use crate::control::bias_channel::{VapoConfig};
use crate::interface::{EvolverEngine, ActionDecoder, CorrectionRequest};
use crate::dsl::schema::ProofAction;

mod dsl;
mod control;
mod interface;

// =========================================================================
// Python 适配器：模拟解码器
// =========================================================================
// 因为 Python 那边传过来的是 Logits，我们需要一个临时的解码策略。
// 在实际深度集成中，这里可以回调 Python 函数，但为了性能，
// 我们通常在 Rust 里实现一个简单的 Top-K 或 Argmax。

struct SimpleRustDecoder {
    vocab_size: usize,
}

impl ActionDecoder for SimpleRustDecoder {
    fn action_space_size(&self) -> usize {
        self.vocab_size
    }

    fn decode(&self, logits: &[f64]) -> ProofAction {
        // 简单的 Argmax 实现
        let max_idx = logits.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        // 注意：这里是一个简化。
        // 实际上我们需要把 Index 映射回具体的 ProofAction。
        // 为了演示 PyO3，我们假设 Index 0 是 Define(Odd), Index 1 是 Define(Even) 等等。
        // 真实场景下，这里应该查表。
        if max_idx == 0 {
             ProofAction::Define { 
                 symbol: "entity".to_string(), 
                 hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
             }
        } else {
             ProofAction::QED
        }
    }
}

// =========================================================================
// Python 类导出：PyEvolver
// =========================================================================

#[pyclass]
struct PyEvolver {
    inner: EvolverEngine,
    action_size: usize,
}

#[pymethods]
impl PyEvolver {
    #[new]
    fn new(action_size: usize) -> Self {
        // 初始化 Rust 引擎
        let config = VapoConfig {
            max_iterations: 50,
            initial_temperature: 1.5,
            valuation_decay: 0.9,
        };
        PyEvolver {
            inner: EvolverEngine::new(Some(config)),
            action_size,
        }
    }

    /// 核心方法：接收 Logits (List[float])，返回修正后的 JSON 字符串
    fn align(&mut self, logits: Vec<f64>) -> PyResult<String> {
        let decoder = SimpleRustDecoder { vocab_size: self.action_size };
        
        let request = CorrectionRequest {
            base_logits: logits,
            request_id: "py_req".to_string(),
        };

        match self.inner.align_generation(request, &decoder) {
            Ok(response) => {
                // 把结果序列化成 JSON 返回给 Python
                // 这样 Python 就可以直接用 json.loads() 解析了
                serde_json::to_string(&response)
                    .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
            },
            Err(e) => Err(PyValueError::new_err(e)),
        }
    }
    
    /// 注入上下文 (从 JSON 字符串)
    fn inject_context(&mut self, action_json: String) -> PyResult<()> {
        let action: ProofAction = serde_json::from_str(&action_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;
        
        self.inner.inject_context(&action);
        Ok(())
    }
}

// =========================================================================
// 模块定义
// =========================================================================
#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    Ok(())
}
