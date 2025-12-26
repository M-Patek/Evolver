// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::{Integer, ops::Pow};
use serde::{Serialize, Deserialize};
use blake3::Hasher;

/// 🏛️ ClassGroupElement: 虚二次域类群元素
/// 表示形式为二元二次型 (a, b, c)，满足 b^2 - 4ac = Delta
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassGroupElement {
    pub a: Integer,
    pub b: Integer,
    pub c: Integer,
}

impl ClassGroupElement {
    /// 构造单位元 (Identity Element)
    /// 对于判别式 D，单位元通常是 (1, 1, (1-D)/4)
    pub fn identity(discriminant: &Integer) -> Self {
        let one = Integer::from(1);
        let four = Integer::from(4);
        // HTP 保证 discriminant = 1 mod 4，所以这里是安全的整数除法
        let c = (one.clone() - discriminant) / &four;
        ClassGroupElement { a: one.clone(), b: one, c }
    }

    /// 🛡️ [Security]: Safe Generator Selection (SGS)
    /// 生成一个密码学安全的、非小阶的生成元。
    pub fn generator(discriminant: &Integer) -> Self {
        let four = Integer::from(4);
        let mut hasher = Hasher::new();
        hasher.update(b"HTP_GENERATOR_SEED_V1");
        hasher.update(&discriminant.to_digits(rug::integer::Order::Lsf));
        let hash_output = hasher.finalize();
        
        let mut p = Integer::from_digits(hash_output.as_bytes(), rug::integer::Order::Lsf);
        p.next_prime_mut();

        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 10_000;

        loop {
            if attempts > MAX_ATTEMPTS {
                panic!("❌ Critical Error: Unable to find valid generator. Discriminant may be flawed."); 
            }

            // 勒让德符号检测 (Delta/p) = 1
            let symbol = discriminant.jacobi(&p);
            if symbol == 1 {
                let modulus = &p * &four;
                let mut b = Integer::from(1);
                
                let b_limit = if &p < &Integer::from(10_000) { &modulus } else { &Integer::from(20_000) };
                let mut found_b = false;
                
                while &b < b_limit {
                    let sq_b = b.clone() * &b;
                    if (sq_b - discriminant).is_divisible(&modulus) {
                        found_b = true;
                        break;
                    }
                    b += 2; // b 必须是奇数
                }

                if found_b {
                    match Self::reduce_form(p.clone(), b, discriminant) {
                        Ok(candidate) => {
                            // Critical: Real Small Order Filter (过滤小阶元素)
                            // 避免陷入 "Kernel Trap"
                            if !candidate.has_small_order(discriminant, 1000) {
                                return candidate;
                            }
                        },
                        Err(_) => {}
                    }
                }
            }
            p.next_prime_mut();
            attempts += 1;
        }
    }

    /// 🛡️ [SECURITY UPGRADE]: 真正的小阶元素检测
    fn has_small_order(&self, discriminant: &Integer, limit_val: u32) -> bool {
        let identity = Self::identity(discriminant);
        
        if self == &identity { return true; }
        if self.a == self.b || self.a == self.c || self.b == 0 { return true; }
        
        let mut annihilator = Integer::from(1);
        let mut p = Integer::from(2);
        let limit = Integer::from(limit_val); 
        
        while &p < &limit {
            annihilator *= &p;
            p.next_prime_mut();
        }

        match self.pow(&annihilator, discriminant) {
            Ok(res) => {
                if res == identity {
                    return true;
                }
                false
            },
            Err(_) => true,
        }
    }

    /// 🌀 State Streaming Evolution (流式演化)
    /// S_new = S_old^p * q
    pub fn apply_affine(&self, p: &Integer, q: &Self, discriminant: &Integer) -> Result<Self, String> {
        let s_powered = self.pow(p, discriminant)?;
        let s_new = s_powered.compose(q, discriminant)?;
        Ok(s_new)
    }

    /// ✨ [FIXED] Composition Algorithm (Cohen Algo 5.4.7)
    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        let s = (&self.b + &other.b) >> 1; 
        
        // Solve: u*a1 + v*a2 = d
        let (d, _u, v) = Self::extended_gcd(&self.a, &other.a);
        
        let a1 = &self.a;
        let a2 = &other.a;
        
        // [FALSIFIABILITY POINT 1]: Composition Compatibility
        // 检查 d | s 是否成立。如果不成立，说明这两个形式无法合成。
        let (_q_dummy, r) = s.div_rem_ref(&d).into();
        if r != Integer::from(0) {
            return Err(format!("Composition Error: gcd(a1, a2)={} does not divide s (s={}). Forms are incompatible.", d, s));
        }
        
        // Compute new A coefficient
        // A = a1 * a2 / d^2
        let a1_div_d = Integer::from(a1 / &d);
        let a2_div_d = Integer::from(a2 / &d);
        let new_a = Integer::from(&a1_div_d * &a2_div_d);

        // Compute new B coefficient
        let s_minus_b2 = &s - &other.b;
        let val = &v * (&s_minus_b2 / &d); 
        let mod_a1_d = &a1_div_d;
        
        let mut k = val;
        k.rem_assign(mod_a1_d);
        if k < 0 { k += mod_a1_d; }

        let term = Integer::from(2) * &a2_div_d * &k;
        let new_b = &other.b + &term;

        // [SECURITY CHECK]: 通过 reduce_form 进行最终的边界验证
        Self::reduce_form(new_a, new_b, discriminant)
    }

    /// ✨ [FIXED] Square Algorithm (NUDUPL / Doubling)
    pub fn square(&self, discriminant: &Integer) -> Result<Self, String> {
        let (g, _x, y) = Self::extended_gcd(&self.a, &self.b);

        let a_div_g = Integer::from(&self.a / &g);
        let new_a = Integer::from(&a_div_g * &a_div_g);

        let target_mod = &a_div_g;
        let mut yc = Integer::from(&y * &self.c);
        yc.rem_assign(target_mod);
        if yc < 0 { yc += target_mod; }

        let term = Integer::from(2) * &a_div_g * &yc;
        let new_b = &self.b + &term;

        Self::reduce_form(new_a, new_b, discriminant)
    }

    /// 🛡️ [Security]: Constant-Sequence Exponentiation (常数序列求幂)
    pub fn pow(&self, exp: &Integer, discriminant: &Integer) -> Result<Self, String> {
        if exp == &Integer::from(0) {
            return Ok(Self::identity(discriminant));
        }
        
        let mut r0 = Self::identity(discriminant);
        let mut r1 = self.clone();
        let bits_count = exp.significant_bits();

        for i in (0..bits_count).rev() {
            let bit = exp.get_bit(i);
            if !bit {
                r1 = r0.compose(&r1, discriminant)?;
                r0 = r0.square(discriminant)?;
            } else {
                r0 = r0.compose(&r1, discriminant)?;
                r1 = r1.square(discriminant)?;
            }
        }
        Ok(r0)
    }

    fn extended_gcd(a: &Integer, b: &Integer) -> (Integer, Integer, Integer) {
        let (mut r0, mut r1) = (a.clone(), b.clone());
        let (mut s0, mut s1) = (Integer::from(1), Integer::from(0));
        let (mut t0, mut t1) = (Integer::from(0), Integer::from(1));

        while r1 != 0 {
            let (q, r2) = r0.div_rem(r1.clone());
            let s2 = s0 - &q * &s1;
            let t2 = t0 - &q * &t1;
            r0 = r1; r1 = r2;
            s0 = s1; s1 = s2;
            t0 = t1; t1 = t2;
        }
        (r0, s0, t0) 
    }

    /// 🛡️ [SECURITY CORE]: 增强型 Reduce Form (The Invariant Fortress)
    /// 包含严格的不变量检查和整除性断言。这是系统的“最高法院”。
    fn reduce_form(mut a: Integer, mut b: Integer, discriminant: &Integer) -> Result<Self, String> {
        let four = Integer::from(4);
        
        // [FALSIFIABILITY POINT 2]: Structural Integrity Check
        // a cannot be zero. A quadratic form with a=0 is degenerate.
        let mut two_a = Integer::from(2) * &a;
        if two_a == 0 { return Err("Math Error: 'a' coefficient is zero (Degenerate Form).".to_string()); }

        // 1. Initial Normalization of b
        b = b.rem_euc(&two_a);
        if b > a { b -= &two_a; }

        // [FALSIFIABILITY POINT 3]: Divisibility Check (The Integral Check)
        // c = (b^2 - D) / 4a. Must be exact integer division.
        // If not, the triplet (a, b, c) does not belong to the discriminant D.
        let numerator = b.clone().pow(2) - discriminant;
        let denominator = &four * &a;
        
        let (c_val, rem) = numerator.div_rem_ref(&denominator).into();
        if rem != Integer::from(0) {
            return Err(format!(
                "Invariant Violated: (b^2 - D) not divisible by 4a. Remainder: {}. \
                This implies the form does not belong to the discriminant group.", 
                rem
            ));
        }
        let mut c = c_val;

        // 2. Reduction Loop with Divergence Protection
        let mut safety_counter = 0;
        const MAX_STEPS: usize = 2000;

        while a > c || (a == c && b < Integer::from(0)) {
            // [FALSIFIABILITY POINT 4]: Algorithmic Convergence
            if safety_counter > MAX_STEPS { 
                return Err("Critical Error: Reduction loop diverged (Infinite Loop Risk / CPU DoS).".to_string());
            }
            
            let num = &c + &b;
            let den = Integer::from(2) * &c;
            if den == 0 { return Err("Math Error: Division by zero in reduction (c=0).".to_string()); }

            let s = num.div_floor(&den); 
            
            let b_new = Integer::from(2) * &c * &s - &b;
            let a_new = c.clone();
            
            // Re-calculate c_new with safety checks
            let num_new = b_new.clone().pow(2) - discriminant;
            let den_new = &four * &a_new;
            
            if den_new == 0 { return Err("Math Error: Division by zero in reduction step.".to_string()); }

            // [FALSIFIABILITY POINT 5]: Intermediate Consistency
            let (c_new_val, rem_new) = num_new.div_rem_ref(&den_new).into();
            if rem_new != Integer::from(0) {
                 return Err("Invariant Violated: Consistency lost during reduction step.".to_string());
            }

            a = a_new; b = b_new; c = c_new_val;
            safety_counter += 1;
        }

        // 3. [FALSIFIABILITY POINT 6]: Final Security Post-Mortem
        // Check A: Discriminant Consistency (b^2 - 4ac == D)
        let check_d = b.clone().pow(2) - Integer::from(4) * &a * &c;
        if &check_d != discriminant {
             return Err(format!("Fatal Logic Error: Result discriminant mismatch. Got {}, Expected {}", check_d, discriminant));
        }
        
        // Check B: Primitive Form (gcd(a, b, c) == 1)
        // 在类群中，我们只处理 Primitive Forms。
        let gcd_ab = a.clone().gcd(&b);
        let gcd_abc = gcd_ab.gcd(&c);
        if gcd_abc != Integer::from(1) {
             return Err(format!("Security Halt: Form is not primitive (gcd={}). Potential attack vector or non-invertible ideal.", gcd_abc));
        }

        Ok(ClassGroupElement { a, b, c })
    }
}
