use num_bigint::{BigInt, Sign};
use num_traits::{Signed, Zero, One, Num};
use num_integer::Integer;
use serde::{Serialize, Deserialize};
use std::mem;
use sha2::{Sha256, Digest}; // [Security Upgrade] 引入 SHA-256

/// ClassGroupElement (类群元素)
/// Represents a binary quadratic form (a, b, c) corresponding to ax^2 + bxy + cy^2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassGroupElement {
    pub a: BigInt,
    pub b: BigInt,
    pub c: BigInt,
}

// 基础的相等性比较
impl PartialEq for ClassGroupElement {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c
    }
}

impl Eq for ClassGroupElement {}

impl ClassGroupElement {
    /// 构造一个新的类群元素
    pub fn new(a: BigInt, b: BigInt, c: BigInt) -> Self {
        Self { a, b, c }
    }

    /// 获取判别式 Δ = b^2 - 4ac
    pub fn discriminant(&self) -> BigInt {
        (&self.b * &self.b) - (BigInt::from(4) * &self.a * &self.c)
    }

    /// 从上下文生成初始种子 (Inception)
    /// 
    /// [Security Upgrade]:
    /// 使用 SHA-256 确保 Context -> Seed 的映射是抗碰撞的。
    /// 这是 Proof of Will 的信任根基。
    pub fn from_context(context: &str, delta: &BigInt) -> (Self, String) {
        let four = BigInt::from(4);
        
        // Step 1: 密码学哈希 (SHA-256)
        let mut hasher = Sha256::new();
        hasher.update(context.as_bytes());
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);
        
        // 将 hash bytes 转为 BigInt 作为初始熵
        let seed_bigint = BigInt::from_bytes_be(Sign::Plus, &hash_result);

        // Step 2: 扩展到大整数 (BigInt Expansion)
        // 使用简单的伪随机扩展，以 seed_bigint 为种子
        let bit_size = delta.bits(); 
        let mut b_expanded = seed_bigint;
        
        // 这里的常数用于扩展位宽，模拟 PRNG
        let multiplier = BigInt::from_str_radix("5DEECE66D", 16).unwrap();
        let increment = BigInt::from(11u32);
        
        while b_expanded.bits() < bit_size {
            b_expanded = (&b_expanded * &multiplier) + &increment;
        }
        
        // 确保 b 的正负号随机性 (取哈希最后一位)
        if hash_result[31] % 2 == 1 {
            b_expanded = -b_expanded;
        }

        // Step 3: 构造合法形式 (a, b, 1)
        // 3.1 调整奇偶性：b^2 ≡ Δ (mod 4)
        let delta_is_even = (&delta % 2).is_zero();
        let b_is_even = (&b_expanded % 2).is_zero();

        if delta_is_even != b_is_even {
            b_expanded += BigInt::one();
        }

        // 3.2 计算 a = (b^2 - Δ) / 4
        let b_sq = &b_expanded * &b_expanded;
        let num = b_sq - delta;
        
        debug_assert!(&num % &four == BigInt::zero());
        
        let a = num / &four;
        let c = BigInt::one();

        // 3.3 构造并约简
        let mut element = Self::new(a, b_expanded, c);
        element.reduce();

        // 返回 (种子状态, 上下文哈希)
        (element, hash_hex)
    }

    /// 高斯合成算法 (Gaussian Composition) - 严格模式
    pub fn compose(&self, other: &Self) -> Self {
        let delta = self.discriminant();
        // 生产环境可优化 panic 为 Result，但 PoW 核心逻辑不允许错误
        if delta != other.discriminant() {
            panic!("CRITICAL MATH VIOLATION: Discriminant mismatch.");
        }

        let two = BigInt::from(2);

        // 1. Unification
        let s = (&self.b + &other.b) / &two;
        let n = (&self.b - &other.b) / &two;

        // 2. Extended GCD
        let egcd1 = self.a.extended_gcd(&other.a);
        let d1 = egcd1.gcd;
        let v = egcd1.y;

        let egcd2 = d1.extended_gcd(&s);
        let d = egcd2.gcd;
        let big_u = egcd2.x;
        let big_v = egcd2.y;

        // 3. 计算 a3
        let d_sq = &d * &d;
        let a1_a2 = &self.a * &other.a;
        let a3 = &a1_a2 / &d_sq;

        // 4. 计算 b3
        let term1 = &big_v * &n;
        let term2 = &big_u * &v * &other.c;
        let big_k = term1 - term2;
        let factor = &two * &other.a / &d;
        let b3_raw = &other.b + &factor * &big_k;

        let two_a3 = &two * &a3;
        let b3 = b3_raw.rem_euclid(&two_a3); 

        // 5. 计算 c3
        let b3_sq = &b3 * &b3;
        let num = &b3_sq - &delta;
        let four_a3 = &two * &two_a3;
        let c3 = num / four_a3;

        // 6. 约简
        let mut result = ClassGroupElement::new(a3, b3, c3);
        result.reduce(); 

        result
    }

    pub fn inverse(&self) -> Self {
        let mut res = ClassGroupElement::new(self.a.clone(), -&self.b, self.c.clone());
        res.reduce();
        res
    }

    pub fn identity(discriminant: &BigInt) -> Self {
        let zero = BigInt::zero();
        let one = BigInt::one();
        let four = BigInt::from(4);

        let rem = discriminant.rem_euclid(&four);

        let (a, b, c) = if rem == zero {
            let c_val = -discriminant / &four;
            (one, zero, c_val)
        } else if rem == one {
            let c_val = (&one - discriminant) / &four;
            (one.clone(), one, c_val)
        } else {
            panic!("Invalid discriminant: must be 0 or 1 mod 4");
        };

        let mut res = ClassGroupElement::new(a, b, c);
        res.reduce();
        res
    }

    /// 高斯约简算法 (Gaussian Reduction)
    fn reduce(&mut self) {
        let zero = BigInt::zero();

        loop {
            let two_a = &self.a << 1; 
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

            if self.a == self.c || self.a == self.b.abs() {
                if self.b < zero {
                    self.b = -&self.b;
                }
            }
            break;
        }
    }
}
