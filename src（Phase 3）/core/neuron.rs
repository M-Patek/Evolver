// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::core::affine::AffineTuple;
use crate::topology::tensor::HyperTensor;
use crate::net::wire::HtpResponse; 
use rug::Integer;
use std::sync::{Arc, RwLock};

/// HTPNeuron: 仿射神经元 (The Processor)
pub struct HTPNeuron {
    pub p_weight: Integer,
    pub memory: Arc<RwLock<HyperTensor>>,
    pub discriminant: Integer,
}

impl HTPNeuron {
    pub fn new(semantic_fingerprint: Integer, dim: usize, side_len: usize, discriminant: Integer) -> Self {
        let tensor = HyperTensor::new(dim, side_len, discriminant.clone());
        HTPNeuron {
            p_weight: semantic_fingerprint,
            memory: Arc::new(RwLock::new(tensor)),
            discriminant,
        }
    }

    pub fn activate(
        &self, 
        input_stream: Vec<AffineTuple>, 
        recursion_depth: usize 
    ) -> Result<(AffineTuple, HtpResponse), String> {
        
        let mut memory_guard = self.memory.write().map_err(|_| "Lock poisoned")?;
        
        // 1. [Non-Commutative Evolution]
        for (t, tuple) in input_stream.iter().enumerate() {
            // (a) 加权
            let weighted_tuple = self.evolve_tuple(tuple, &self.p_weight)?;

            // (b) 注入时空噪声
            let time_noise = self.generate_spacetime_noise(t)?;
            let evolved = weighted_tuple.compose(&time_noise, &self.discriminant)?;

            // (c) 写入内部记忆张量
            // [UPDATE]: 传入时间戳 t，开启 Spacetime Orthogonal Storage
            let coord_str = format!("seq:{}", t);
            memory_guard.insert(&coord_str, evolved, t as u64)?;
        }

        // 2. [Fold]
        // 现在这里会先触发 Micro-Fold，再触发 Macro-Fold
        let raw_output = memory_guard.calculate_global_root()?;

        // 3. [Reduce]
        let final_output = self.algebraic_reduction(raw_output, recursion_depth)?;

        // 4. [Proof Generation]
        let proof_coord = memory_guard.map_id_to_coord(0); 
        let proof_path = memory_guard.get_segment_tree_path(&proof_coord, 0);
        
        let proof = HtpResponse::ProofBundle {
            request_id: 0,
            primary_path: proof_path,
            orthogonal_anchors: vec![],
            epoch: recursion_depth as u64,
        };

        Ok((final_output, proof))
    }

    fn evolve_tuple(&self, tuple: &AffineTuple, weight: &Integer) -> Result<AffineTuple, String> {
        let new_p = Integer::from(&tuple.p_factor * weight);
        let new_q = tuple.q_shift.pow(weight, &self.discriminant)?;
        
        Ok(AffineTuple {
            p_factor: new_p,
            q_shift: new_q,
        })
    }

    fn generate_spacetime_noise(&self, t: usize) -> Result<AffineTuple, String> {
        let g = crate::core::algebra::ClassGroupElement::generator(&self.discriminant);
        let h_t = Integer::from(t + 1);
        let q_noise = g.pow(&h_t, &self.discriminant)?;
        
        Ok(AffineTuple {
            p_factor: Integer::from(1),
            q_shift: q_noise,
        })
    }

    fn algebraic_reduction(&self, tuple: AffineTuple, depth: usize) -> Result<AffineTuple, String> {
        let identity = AffineTuple::identity(&self.discriminant);
        if depth > 10 {
             return tuple.compose(&identity, &self.discriminant);
        }
        Ok(tuple)
    }
}
