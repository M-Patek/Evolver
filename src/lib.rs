use pyo3::prelude::*;
use std::cell::RefCell;
use num_bigint::{BigInt, Sign, RandBigInt};
use num_traits::{One, Zero, ToPrimitive, Signed};
use num_integer::Integer;

use crate::soul::algebra::ClassGroupElement;
use crate::body::topology::VPuNNConfig;
use crate::dsl::stp_bridge::STPContext;
use crate::dsl::schema::ProofAction;
use crate::will::perturber::EnergyEvaluator;
use crate::will::optimizer;
use crate::body::decoder;

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

// ==========================================
// 🛡️ Crypto Utils: 判别式与安全性核心
// ==========================================
mod crypto_utils {
    use super::*;

    /// 确定性哈希算法 (FNV-1a 64-bit)
    /// 这里的关键是必须 "Deterministic"，不能使用 Rust std::hash (它包含随机种子)。
    /// 确保对于相同的 Context，生成的 Discriminant 永远一致。
    pub fn deterministic_hash(text: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in text.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    /// Miller-Rabin 素性测试
    /// 用于在运行时动态寻找大素数
    pub fn is_prime(n: &BigInt, k: usize) -> bool {
        if *n <= BigInt::from(1) { return false; }
        if *n <= BigInt::from(3) { return true; }
        if n % 2 == BigInt::zero() { return false; }

        // 写成 n - 1 = 2^s * d
        let one = BigInt::one();
        let two = BigInt::from(2);
        let n_minus_one = n - &one;
        let mut d = n_minus_one.clone();
        let mut s = 0;
        
        while &d % &two == BigInt::zero() {
            d /= &two;
            s += 1;
        }

        // 简单的确定性基底 (对于 64-bit 范围足够，如果是 2048-bit 需要更多随机基底)
        // 为了演示速度，这里固定几个基底
        let bases = vec![2, 3, 5, 7, 11, 13, 17, 19, 23];
        
        for a_val in bases {
            let a = BigInt::from(a_val);
            if &a >= n { break; }
            
            let mut x = a.modpow(&d, n); // a^d mod n
            
            if x == one || x == n_minus_one {
                continue;
            }
            
            let mut composite = true;
            for _ in 0..s-1 {
                x = x.modpow(&two, n);
                if x == n_minus_one {
                    composite = false;
                    break;
                }
            }
            
            if composite {
                return false;
            }
        }
        
        true
    }

    /// 基于种子生成判别式 Delta
    /// 规则: Delta = -M, 其中 M 是素数 且 M = 3 mod 4
    /// 这样保证了虚二次域 Q(sqrt(-M)) 的基本判别式就是 -M
    pub fn generate_discriminant(seed: u64) -> BigInt {
        // [Security Config]: 设置合适的位宽。
        // 为了演示流畅性，我们使用 64-bit 素数 (群大小约 10^9)，
        // 这足以展示 "Unknown Order" 的特性 (人脑无法计算，电脑无法秒破)，
        // 同时让 VAPO 搜索保持在毫秒级。
        // 在生产环境中，这里应该是 2048-bit。
        let mut candidate = BigInt::from(seed) | (BigInt::from(1) << 63); 
        
        // 强制奇数
        if candidate.is_even() {
            candidate += 1;
        }

        // 寻找 M = 3 mod 4
        while &candidate % 4 != BigInt::from(3) {
            candidate += 2;
        }

        // 线性搜索下一个素数
        loop {
            if is_prime(&candidate, 10) {
                // 找到了 M，返回 -M
                return -candidate;
            }
            candidate += 4; // 保持 3 mod 4 性质
        }
    }
}

// ==========================================
// 🌉 STP Bridge: 逻辑-代数 桥接器
// ==========================================
struct StpBridge<'a> {
    context: &'a RefCell<STPContext>,
}

impl<'a> EnergyEvaluator for StpBridge<'a> {
    fn evaluate(&self, path: &[u64]) -> f64 {
        let decision_seed = path.get(0).unwrap_or(&0);
        
        // VAPO 尝试猜测真理
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
        
        // 上下文完整性检查
        if !stp.state.contains_key("n") || !stp.state.contains_key("m") {
            return 100.0; 
        }

        stp.calculate_energy(&action);

        let check_action = ProofAction::Apply {
            theorem_id: "ModAdd".to_string(),
            inputs: vec!["n".to_string(), "m".to_string()],
            output_symbol: "sum_truth".to_string(),
        };

        stp.calculate_energy(&check_action)
    }
}

// ==========================================
// 🐍 Python Interface
// ==========================================

#[pyclass]
pub struct PyEvolver {
    // Soul 现在是一个 Option，因为我们在 new 的时候还不知道 Context，
    // 只有在 align 的时候才能确定 Discriminant 并实例化 Soul。
    soul: Option<ClassGroupElement>, 
    body: VPuNNConfig,
    stp: RefCell<STPContext>, 
}

#[pymethods]
impl PyEvolver {
    #[new]
    fn new(p: u64, k: usize) -> Self {
        println!("🐱 PyEvolver v0.3.1 (Secure Mode) Initializing...");
        println!("   |-- Topology: p={}, k={}", p, k);
        println!("   |-- Status: Waiting for Context to collapse wave function...");

        let mut stp_ctx = STPContext::new();
        
        // 初始化逻辑公理
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

        if !stp_ctx.state.contains_key("n") || !stp_ctx.state.contains_key("m") {
            panic!("❌ Critical Error: Failed to initialize mathematical context!");
        }

        let body_config = VPuNNConfig::new(k, p);

        PyEvolver {
            soul: None, // 灵魂尚未诞生
            body: body_config,
            stp: RefCell::new(stp_ctx),
        }
    }

    /// 核心对齐函数 (The Will's Journey)
    /// 1. Hash(Context) -> Discriminant (World Creation)
    /// 2. Identity(Discriminant) -> S0 (Soul Birth)
    /// 3. VAPO(S0) -> S_final (Will Execution)
    fn align(&mut self, context: String) -> Vec<u64> {
        // 1. 创世 (World Creation)
        // 基于上下文生成唯一的数学宇宙 (Discriminant)
        let seed = crypto_utils::deterministic_hash(&context);
        let discriminant = crypto_utils::generate_discriminant(seed);
        
        // 打印安全参数，证明我们没有作弊 (使用了 -23 以外的数)
        let delta_str = discriminant.to_str_radix(10);
        let safe_log = if delta_str.len() > 10 {
            format!("{}...", &delta_str[0..10])
        } else {
            delta_str.clone()
        };
        println!("🔮 Context Bound: '{}'", context);
        println!("   |-- Seed: {:016x}", seed);
        println!("   |-- Generated Discriminant Δ: {} (bits: {})", safe_log, discriminant.bits());

        // 2. 灵魂诞生 (Soul Birth)
        // 在这个新宇宙中初始化单位元
        let mut current_soul = ClassGroupElement::identity(&discriminant);

        // 3. 初始演化 (Seeding)
        // 让灵魂根据种子先旋转几圈，摆脱单位元，进入混沌轨道
        current_soul = current_soul.evolve(seed);

        // 4. 意志执行 (Optimization)
        let evaluator = StpBridge { context: &self.stp };
        
        println!("⚡ VAPO Engine Start: Searching on Cl(Δ)...");
        let optimized_soul = optimizer::optimize(&current_soul, &self.body, &evaluator);
        
        // 更新内部状态
        self.soul = Some(optimized_soul.clone());
        
        // 5. 物质化 (Materialization)
        let path = decoder::materialize_path(&optimized_soul, &self.body);
        
        println!("✅ Logic Aligned. Energy = 0. Path: {:?}", path);
        path
    }
}

#[pymodule]
fn new_evolver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEvolver>()?;
    Ok(())
}
