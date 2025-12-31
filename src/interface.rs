// src/lib.rs
// Rust 和 Python 的“外交公署” v0.3 (Lite Edition)
// 适配 BiasChannel v0.2 API，但移除了 HTP 密码学组件
// 专注于 Phase 1: 纯逻辑纠偏

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

use crate::control::bias_channel::{VapoConfig};
use crate::interface::{EvolverEngine, ActionDecoder, CorrectionRequest};
use crate::dsl::schema::ProofAction;

// 模块声明：只保留逻辑核心
mod dsl;
mod control;
mod interface;
// mod crypto; // [Removed] 密码学模块已剥离

// =========================================================================
// Python 适配器：模拟解码器
// =========================================================================

struct SimpleRustDecoder {
    vocab_size: usize,
}

impl ActionDecoder for SimpleRustDecoder {
    fn action_space_size(&self) -> usize {
        self.vocab_size
    }

    fn decode(&self, logits: &[f64]) -> ProofAction {
        // 简单的 Argmax 实现
        // 在真实场景中，这里会将 logits 映射回 Token ID，再解析为 Action JSON
        let max_idx = logits.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        // [Mock] 简化的词表映射用于演示
        // 0 -> 错误的定义 (Odd)
        // 其他 -> 正确的定义 (Even) 或 QED
        if max_idx == 0 {
             ProofAction::Define { 
                 symbol: "sum_truth".to_string(), 
                 hierarchy_path: vec!["Odd".to_string()] 
             }
        } else if max_idx == 1 {
             ProofAction::Define { 
                 symbol: "sum_truth".to_string(), 
                 hierarchy_path: vec!["Even".to_string()] 
             }
        } else {
             ProofAction::QED
        }
    }
}

// =========================================================================
// [Removed] Crypto 适配器区域 (PyClassGroup, py_hash_to_prime...)
// =========================================================================

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
        // 默认 VAPO 配置，专为逻辑纠偏优化
        let config = VapoConfig {
            max_iterations: 50,      // 快速搜索限制
            initial_temperature: 1.5, // 较高的初始温度以跳出局部最优
            valuation_decay: 0.9,     // 快速冷却
        };
        PyEvolver {
            inner: EvolverEngine::new(Some(config)),
            action_size,
        }
    }

    /// 执行逻辑对齐
    /// Python 调用示例: 
    /// evolver.align(logits_list, "Prove sum of two Odds is Even")
    fn align(&mut self, logits: Vec<f64>, context: String) -> PyResult<String> {
        let decoder = SimpleRustDecoder { vocab_size: self.action_size };
        
        // [Lite Mode] 使用简单的非加密哈希生成种子，仅用于重现性
        // 既然不需要密码学防御，我们用简单的随机数或 Context 哈希即可
        let mut seed_gen = 0xCAFEBABE;
        for byte in context.bytes() {
            seed_gen = seed_gen.wrapping_add(byte as u64).wrapping_mul(31);
        }

        let request = CorrectionRequest {
            base_logits: logits,
            request_id: "py_req_lite".to_string(),
            context: context,
            seed: seed_gen, 
        };

        match self.inner.align_generation(request, &decoder) {
            Ok(response) => {
                // 返回 JSON 格式的纠偏结果
                serde_json::to_string(&response)
                    .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
            },
            Err(e) => Err(PyValueError::new_err(e)),
        }
    }
    
    /// 注入已知事实或前置条件到 STP 引擎
    fn inject_context(&mut self, action_json: String) -> PyResult<()> {
        let action: ProofAction = serde_json::from_str(&action_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;
        
        self.inner.inject_context(&action);
        Ok(())
    }

    /// 重置 STP 状态机 (清空上下文)
    fn reset(&mut self) {
        self.inner.reset();
    }
}

// =========================================================================
// 模块定义
// =========================================================================
#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    // m.add_class::<PyClassGroup>()?; // [Removed]
    // m.add_function(wrap_pyfunction!(py_hash_to_prime, m)?)?; // [Removed]
    Ok(())
}
