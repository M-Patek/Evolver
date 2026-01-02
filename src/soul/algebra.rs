use num_bigint::{BigInt, Sign};
use num_traits::{Signed, Zero, One, Num, ToPrimitive};
use num_integer::Integer;
use serde::{Serialize, Deserialize};
use std::mem;
use sha2::{Sha256, Digest};
use std::fmt;
use thiserror::Error; // 引入 thiserror 喵！

/// Evolver 系统错误定义
/// 这里的错误是可以被 VAPO 捕捉并转化为能量惩罚的
#[derive(Error, Debug, Clone, PartialEq)]
pub enum EvolverError {
    #[error("Cosmic Mismatch (Universe Violation): {0} vs {1}")]
    CosmicMismatch(String, String),
    
    #[error("Singularity: Division by zero detected")]
    Singularity,
    
    #[error("Numerical Instability: Result became NaN or Infinite")]
    NumericalInstability,

    #[error("Invalid State: {0}")]
    InvalidState(String),
}

/// 理想类 (Ideal Class)
/// 代表虚二次域 Cl(Δ) 中的二元二次型 (a, b, c) -> ax^2 + bxy + cy^2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdealClass {
    pub a: BigInt,
    pub b: BigInt,
    pub c: BigInt,
}

// 基础相等性比较
impl PartialEq for IdealClass {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c
    }
}
impl Eq for IdealClass {}

impl fmt::Display for IdealClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.a, self.b, self.c)
    }
}

/// 宇宙上下文
pub struct Universe {
    pub discriminant: BigInt,
    pub context_hash: String,
}

impl IdealClass {
    pub fn new(a: BigInt, b: BigInt, c: BigInt) -> Self {
        Self { a, b, c }
    }

    /// [Security Check] 验证两个元素是否属于同一个宇宙 (判别式相同)
    /// 现在的返回值是 Result，不再是 Panic 喵！
    pub fn ensure_same_universe(&self, other: &Self) -> Result<(), EvolverError> {
        let delta_self = self.discriminant();
        let delta_other = other.discriminant();
        
        if delta_self != delta_other {
            return Err(EvolverError::CosmicMismatch(
                delta_self.to_string(), 
                delta_other.to_string()
            ));
        }
        Ok(())
    }

    pub fn from_hash(context: &str, _p: u64) -> Self {
        let (seed, _) = Self::spawn_universe(context);
        seed
    }

    /// 自旋演化 (VDF Squaring)
    pub fn square(&self) -> Result<Self, EvolverError> {
        self.compose(self)
    }

    pub fn discriminant(&self) -> BigInt {
        (&self.b * &self.b) - (BigInt::from(4) * &self.a * &self.c)
    }

    pub fn spawn_universe(context: &str) -> (Self, Universe) {
        let target_bits = 2048; 
        let expanded_bytes = expand_entropy(context, target_bits / 8);
        let seed_bigint = BigInt::from_bytes_be(Sign::Plus, &expanded_bytes);

        let mut hasher = Sha256::new();
        hasher.update(context.as_bytes());
        let context_hash = format!("{:x}", hasher.finalize());

        let m_prime = next_prime_3_mod_4(seed_bigint.clone());
        let delta = -m_prime;

        let element = Self::generate_seed_in_delta(&delta, &seed_bigint);

        let universe = Universe {
            discriminant: delta,
            context_hash,
        };

        (element, universe)
    }

    fn generate_seed_in_delta(delta: &BigInt, initial_entropy: &BigInt) -> Self {
        let four = BigInt::from(4);
        let mut b_curr = initial_entropy.clone();
        
        if (&b_curr % 2).is_zero() {
            b_curr += BigInt::one();
        }

        let b_sq = &b_curr * &b_curr;
        let num = b_sq - delta;
        
        let a = num / &four;
        let c = BigInt::one(); 

        let mut element = Self::new(a, b_curr, c);
        element.reduce();
        element
    }

    /// 高斯合成算法 (Gaussian Composition)
    /// [Robustness Fix] 现在返回 Result<Self, EvolverError>
    pub fn compose(&self, other: &Self) -> Result<Self, EvolverError> {
        // 1. 严格性检查
        self.ensure_same_universe(other)?;

        let delta = self.discriminant();
        let two = BigInt::from(2);

        // 2. Unification
        let s = (&self.b + &other.b) / &two;
        let n = (&self.b - &other.b) / &two;

        // 3. Extended GCD
        let egcd1 = self.a.extended_gcd(&other.a);
        let d1 = egcd1.gcd;
        let v = egcd1.y;

        let egcd2 = d1.extended_gcd(&s);
        let d = egcd2.gcd;
        
        // [Safety] 检查除零奇点
        if d.is_zero() {
            return Err(EvolverError::Singularity);
        }

        let big_u = egcd2.x;
        let big_v = egcd2.y;

        // 4. Solve components
        let d_sq = &d * &d;
        let a1_a2 = &self.a * &other.a;
        let a3 = &a1_a2 / &d_sq;

        let term1 = &big_v * &n;
        let term2 = &big_u * &v * &other.c;
        let big_k = term1 - term2;
        let factor = &two * &other.a / &d;
        let b3_raw = &other.b + &factor * &big_k;

        let two_a3 = &two * &a3;
        
        // [Safety] 检查约化基是否为零
        if two_a3.is_zero() {
            return Err(EvolverError::Singularity);
        }

        let b3 = b3_raw.rem_euclid(&two_a3); 

        let b3_sq = &b3 * &b3;
        let num = &b3_sq - &delta;
        let four_a3 = &two * &two_a3;
        
        if four_a3.is_zero() {
            return Err(EvolverError::Singularity);
        }

        let c3 = num / four_a3;

        let mut result = IdealClass::new(a3, b3, c3);
        result.reduce(); 
        Ok(result)
    }

    pub fn inverse(&self) -> Self {
        let mut res = IdealClass::new(self.a.clone(), -&self.b, self.c.clone());
        res.reduce();
        res
    }

    fn reduce(&mut self) {
        let two_a = &self.a << 1; 
        if two_a.is_zero() { return; } // 防止死循环

        loop {
            if self.b.abs() > self.a {
                let mut r = &self.b % &two_a; 
                if r > self.a { r -= &two_a; } 
                else if r <= -&self.a { r += &two_a; }
                
                let b_new = r;
                let k = (&b_new - &self.b) / &two_a; 
                
                let term = &self.b + (&self.a * &k);
                self.c = &self.c + &k * term;
                self.b = b_new;
            }

            if self.a > self.c {
                mem::swap(&mut self.a, &mut self.c);
                self.b = -&self.b;
                continue;
            }

            if self.a == self.b.abs() || self.a == self.c {
                if self.b < BigInt::zero() {
                    self.b = -&self.b;
                }
            }
            
            if self.b.abs() <= self.a && self.a <= self.c {
                break;
            }
        }
    }
}

// --- Helper Functions ---
fn expand_entropy(input: &str, target_bytes: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(target_bytes);
    let mut counter = 0u32;
    while result.len() < target_bytes {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hasher.update(counter.to_be_bytes());
        result.extend_from_slice(&hasher.finalize());
        counter += 1;
    }
    result.truncate(target_bytes);
    result
}

fn next_prime_3_mod_4(mut start: BigInt) -> BigInt {
    if (&start % 2).is_zero() { start += 1; }
    while (&start % 4) != BigInt::from(3) { start += 2; }
    loop {
        if is_probable_prime(&start, 5) { return start; }
        start += 4; 
    }
}

fn is_probable_prime(n: &BigInt, k: u32) -> bool {
    let one = BigInt::one();
    let two = BigInt::from(2);
    if *n <= one { return false; }
    if *n == two || *n == BigInt::from(3) { return true; }
    if (n % &two).is_zero() { return false; }

    let mut d = n - &one;
    let mut s = 0;
    while (&d % &two).is_zero() { d /= &two; s += 1; }
    let mut witness_gen = n.clone(); 
    for _ in 0..k {
        witness_gen = (&witness_gen * BigInt::from(48271u32)) % (n - &BigInt::from(3));
        let a = &witness_gen + &two;
        let mut x = mod_pow(&a, &d, n);
        if x == one || x == n - &one { continue; }
        let mut composite = true;
        for _ in 0..(s - 1) {
            x = mod_pow(&x, &two, n);
            if x == n - &one { composite = false; break; }
        }
        if composite { return false; }
    }
    true
}

fn mod_pow(base: &BigInt, exp: &BigInt, modulus: &BigInt) -> BigInt {
    base.modpow(exp, modulus)
}
