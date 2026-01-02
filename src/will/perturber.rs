use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Signed, Zero, One, ToPrimitive};
use rand::prelude::*;
use crate::soul::algebra::IdealClass;

pub struct Perturber {
    generators: Vec<IdealClass>,
}

impl Perturber {
    pub fn new(discriminant: &BigInt, count: usize) -> Self {
        let generators = generate_perturbations(discriminant, count);
        Self { generators }
    }

    /// [New] 公开生成元集合，供验证者 (Verifier) 重建凯莱图拓扑
    pub fn get_generators(&self) -> &Vec<IdealClass> {
        &self.generators
    }

    /// 返回 (新状态, 施加的扰动元)
    /// 用于 Trace 记录
    pub fn perturb_with_source(&self, state: &IdealClass) -> (IdealClass, IdealClass) {
        if self.generators.is_empty() {
            return (state.clone(), state.clone()); 
        }

        let mut rng = rand::thread_rng();
        let gen = self.generators.choose(&mut rng).unwrap();
        
        if rng.gen_bool(0.5) {
            // S * g
            (state.compose(gen), gen.clone())
        } else {
            // S * g^-1
            let inv = gen.inverse();
            (state.compose(&inv), inv)
        }
    }
    
    pub fn perturb(&self, state: &IdealClass) -> IdealClass {
        self.perturb_with_source(state).0
    }
}

// ... 辅助函数 ...

/// 确定性生成扰动元集合
/// 必须保证相同的 Δ 和 count 生成相同的集合 P
fn generate_perturbations(discriminant: &BigInt, count: usize) -> Vec<IdealClass> {
    let mut perturbations = Vec::with_capacity(count);
    let mut p_candidate = 2u64;
    
    // 为了防止无限循环（如果找不到足够的素理想），设置一个硬上限
    let max_candidate = 100_000; 

    while perturbations.len() < count && p_candidate < max_candidate {
        // 必须是素数
        if is_prime(p_candidate) {
            // 克罗内克符号 (Δ/p) = 1 (分裂) 或 0 (分支) 才有理想
            // 实际上 try_create_prime_form 内部做了这步检查
            if let Some(element) = try_create_prime_form(discriminant, p_candidate) {
                perturbations.push(element);
            }
        }
        p_candidate += 1;
    }
    
    // 如果生成元不足，在生产环境中应该报错，这里为了 Demo 保持容错
    // eprintln!("Warning: Requested {} generators, found {}", count, perturbations.len());
    
    perturbations
}

fn try_create_prime_form(discriminant: &BigInt, p: u64) -> Option<IdealClass> {
    let p_bi = BigInt::from(p);
    let four_p = BigInt::from(4) * &p_bi;
    
    // 我们需要解 b^2 ≡ Δ (mod 4p)
    // 也就是 b^2 = Δ + 4p * k
    // 简化检查: Jacobi symbol (Δ/p)
    
    // 这里使用简化的暴力搜索 b in [0, 2p]
    // 对于小 p 效率很高
    
    let start = if discriminant.is_odd() { 1 } else { 0 };
    let step = 2;
    let limit = 2 * p; // b < 2p 通常足够
    
    let mut b_curr = start;
    while b_curr <= limit {
        let b_bi = BigInt::from(b_curr);
        let b_sq = &b_bi * &b_bi;
        
        // Check condition: b^2 ≡ Δ (mod 4p)
        let diff = &b_sq - discriminant;
        if (&diff % &four_p).is_zero() {
             let c_val = diff / &four_p;
             return Some(IdealClass::new(p_bi.clone(), b_bi, c_val));
        }
        
        // Also check -b (which is just 2p - b or similar in modular arithmetic, but explicit check implies both roots)
        // Note: For IdealClass(a, b, c), the inverse is (a, -b, c). 
        // We only need one representative per prime p. The inverse is implicitly in the Cayley graph edges.
        
        b_curr += step;
    }
    None
}

fn is_prime(n: u64) -> bool {
    if n <= 1 { return false; }
    if n <= 3 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}
