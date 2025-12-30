// src/interface.rs
// Evolver Public Interface

use crate::control::bias_channel::{BiasController, VapoConfig};
use crate::dsl::schema::ProofAction;
use crate::dsl::stp_bridge::STPContext;

// =========================================================================
// 1. 核心特征：解码器适配器 (The Decoder Trait)
// =========================================================================
pub trait ActionDecoder {
    fn decode(&self, logits: &[f64]) -> ProofAction;
    fn action_space_size(&self) -> usize;
}

// =========================================================================
// 2. 输入/输出 结构体 (DTOs)
// =========================================================================

#[derive(Debug, Clone)]
pub struct CorrectionRequest {
    pub base_logits: Vec<f64>,
    pub request_id: String,
}

#[derive(Debug, Clone)]
pub struct CorrectionResponse {
    pub final_action: ProofAction,
    pub applied_bias: Vec<i32>,
    pub final_energy: f64,
    pub iterations: usize,
}

// =========================================================================
// 3. Evolver 引擎实例 (The Engine)
// =========================================================================

// 修复：移除 'pub class'，使用 Rust 标准 struct
pub struct EvolverEngine {
    stp_ctx: STPContext,
    controller: BiasController,
}

impl EvolverEngine {
    pub fn new(config: Option<VapoConfig>) -> Self {
        EvolverEngine {
            stp_ctx: STPContext::new(),
            controller: BiasController::new(config),
        }
    }

    pub fn reset(&mut self) {
        self.stp_ctx = STPContext::new();
        self.controller = BiasController::new(None); 
    }

    pub fn inject_context(&mut self, action: &ProofAction) {
        self.stp_ctx.calculate_energy(action);
    }

    pub fn align_generation<D: ActionDecoder>(
        &mut self, 
        request: CorrectionRequest, 
        decoder: &D
    ) -> Result<CorrectionResponse, String> {
        
        if request.base_logits.len() != decoder.action_space_size() {
            return Err(format!("Logits dimension mismatch: expected {}, got {}", 
                decoder.action_space_size(), request.base_logits.len()));
        }

        let decode_wrapper = |logits: &[f64]| -> ProofAction {
            decoder.decode(logits)
        };

        // 调用 Controller 的优化循环
        let (final_bias, final_action) = self.controller.optimize(
            &request.base_logits, 
            &mut self.stp_ctx, 
            decode_wrapper
        );

        let final_energy = self.stp_ctx.calculate_energy(&final_action);

        Ok(CorrectionResponse {
            final_action,
            applied_bias: final_bias.data,
            final_energy,
            iterations: 0, // TODO: 从 controller 获取实际迭代次数
        })
    }
}
