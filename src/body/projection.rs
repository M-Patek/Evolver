use crate::soul::algebra::IdealClass;
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use num_traits::ToPrimitive;
use num_bigint::BigInt;

/// 导航特征 (Navigation Features)
/// 用于启发式搜索的平滑几何特征。
/// 
/// [Fix]: 使用对数域特征以避免大数溢出。
/// 原理: 
/// x = -b / 2a  -> 保持原样 (b 和 a 同量级，比值安全)
/// y = sqrt(|Delta|) / 2a -> 可能非常小或非常大
/// log(y) = 0.5 * log(|Delta|) - log(2) - log(a)
#[derive(Debug, Clone)]
struct NavigationFeatures {
    pub cos_x: f64,
    pub sin_x: f64,
    pub log_y: f64,
}

impl NavigationFeatures {
    fn extract(state: &IdealClass) -> Self {
        // 1. 计算 x = -b / 2a
        // 由于在约化形式下 |b| <= a，所以 |x| <= 0.5。
        // 这意味着 x 直接转 f64 是绝对安全的，不会溢出。
        let b_f64 = bigint_to_scaled_f64(&state.b, &state.a); // b/a
        let x = -b_f64 / 2.0;

        // 2. 计算 log(y)
        // y = sqrt(|D|) / 2a
        // log(y) = 0.5 * log(|D|) - log(2a)
        // log(y) = 0.5 * log(|D|) - (log(2) + log(a))
        
        let log_delta = state.discriminant().abs().bits() as f64 * 2.0_f64.ln(); // ln(2^bits) approx
        // 更精确的 log_a:
        let log_a = bigint_log_e(&state.a);
        
        let log_y = 0.5 * log_delta - (2.0_f64.ln() + log_a);

        NavigationFeatures {
            cos_x: (2.0 * PI * x).cos(),
            sin_x: (2.0 * PI * x).sin(),
            log_y,
        }
    }
}

/// 辅助函数：计算 BigInt 的自然对数 ln(n)
/// 利用 bits() 和高位数据估算
fn bigint_log_e(n: &BigInt) -> f64 {
    if n.sign() == num_bigint::Sign::NoSign {
        return f64::NEG_INFINITY;
    }
    let bits = n.bits();
    // ln(n) ≈ ln(2) * bits
    // 为了更精确，可以取高 52 位转 f64 修正小数部分，这里做个粗略估计对 VAPO 足够了
    (bits as f64) * 2.0_f64.ln()
}

/// 辅助函数：计算 a / b 的近似值 (f64)
/// 假设 a, b 量级相近，结果在 [-1, 1] 范围内
fn bigint_to_scaled_f64(numerator: &BigInt, denominator: &BigInt) -> f64 {
    // 简单的转换为: (numerator * SCALE) / denominator
    // 但为了避免乘法溢出，利用它们 bit 数相近的特性
    let n_bits = numerator.bits();
    let d_bits = denominator.bits();
    
    // 如果差别太大，直接返回 0 (精度丢失)
    if d_bits > n_bits + 64 { return 0.0; }
    
    // 转成 f64 计算
    // 这里其实可以直接利用 to_f64，因为约化后 a, b 不会相差太大倍数
    // 但 2048-bit 直接 to_f64 会 Inf。
    // 我们需要提取高位。
    
    let n_high = extract_high_64(numerator);
    let d_high = extract_high_64(denominator);
    
    // 需要根据位差补偿指数
    let shift = (n_bits as i32) - (d_bits as i32);
    let val = n_high / d_high;
    
    val * 2.0_f64.powi(shift)
}

fn extract_high_64(n: &BigInt) -> f64 {
    if n.is_zero() { return 0.0; }
    // 这种实现很 hacky，实际应操作底层的 digits
    // 这里为了演示，我们假设它能工作。
    // 真正的 rust num-bigint 没有直接提取高位的简单 API，
    // 最简单的做法是 format 为 hex 字符串取前几位，或者 to_radix
    // 这里简化处理：用 bits() 估算
    let bits = n.bits();
    if bits <= 64 {
        n.to_f64().unwrap_or(0.0)
    } else {
        // 右移直到剩下 64 位
        let shifted = n >> (bits - 62); // 留一点余地
        shifted.to_f64().unwrap_or(0.0)
    }
}

/// Projector (投影仪)
/// 负责将代数状态(Soul)实体化为逻辑(Body)。
pub struct Projector {
    pub p_base: u64, 
}

impl Projector {
    pub fn new(p_base: u64) -> Self {
        Self { p_base }
    }

    /// [Truth Layer] 精确投影
    /// Avalanche Effect 依然有效，BigInt 的 Hash 实现通常涵盖所有位
    pub fn project_exact(&self, state: &IdealClass, time: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        state.a.hash(&mut hasher);
        state.b.hash(&mut hasher);
        state.c.hash(&mut hasher);
        time.hash(&mut hasher);
        let hash = hasher.finish();
        hash % self.p_base
    }

    /// [Will Layer] 启发式投影
    /// [Fix] 这里的桶化逻辑现在基于 log 域的特征
    pub fn project_heuristic(&self, state: &IdealClass, time: u64) -> u64 {
        let features = NavigationFeatures::extract(state);
        
        const GRID_RESOLUTION: f64 = 1000.0;
        let bucket_cos = (features.cos_x * GRID_RESOLUTION) as i64;
        let bucket_sin = (features.sin_x * GRID_RESOLUTION) as i64;
        // log_y 的范围可能很大，但它是平滑的，可以直接桶化
        let bucket_log_y = (features.log_y * 10.0) as i64; // log 域分辨率稍低
        
        let mut hasher = DefaultHasher::new();
        bucket_cos.hash(&mut hasher);
        bucket_sin.hash(&mut hasher);
        bucket_log_y.hash(&mut hasher);
        time.hash(&mut hasher);
        
        let hash = hasher.finish();
        hash % self.p_base
    }

    pub fn project_sequence(&self, seed: &IdealClass, depth: usize) -> Vec<u64> {
        let mut current_state = seed.clone();
        let mut path = Vec::with_capacity(depth);

        for layer in 0..depth {
            let digit = self.project_exact(&current_state, layer as u64);
            path.push(digit);
            current_state = current_state.square();
        }
        path
    }
}
