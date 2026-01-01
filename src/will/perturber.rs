use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use num_traits::{One, Zero, Signed, ToPrimitive};
use crate::soul::algebra::ClassGroupElement;

/// 算法版本常量
///
/// 当修改 generate_perturbations 的内部逻辑时，必须同步修改此版本号。
/// 验证者通过校验此版本号来决定是否接受 ProofBundle。
pub const ALGORITHM_VERSION: &str = "v1_sequential_primes";

/// 能量评估器特质 (Energy Evaluator Trait)
///
/// 该接口定义了 VAPO 如何与 STP 引擎交互。
/// 任何实现了此接口的结构体都能计算给定离散序列（digits）的逻辑能量。
///
/// - `digits`:通常代表通过生成器输出并经过偏置修正后的 Token ID 序列。
/// - 返回值 `f64`: 能量值 $E$。$E=0$ 表示逻辑约束满足，$E>0$ 表示违规。
pub trait EnergyEvaluator {
    fn evaluate(&self, digits: &[u64]) -> f64;
}

/// 生成扰动元生成器
///
/// 该模块负责生成用于 VAPO (Valuation-Adaptive Perturbation Optimization) 的微小扰动。
/// 每一个扰动对应理想类群 $Cl(\Delta)$ 中的一个范数较小的元素。
///
/// 算法原理 (v1_sequential_primes):
/// 1. 遍历小素数 p (2, 3, 5...)。
/// 2. 计算 Kronecker 符号 $(\Delta / p)$。
/// 3. 如果结果为 1，说明该素数在域中分裂，存在范数为 p 的理想类。
/// 4. 求解同余方程 $b^2 \equiv \Delta \pmod{4p}$ 找到对应的 b。
/// 5. 构造形式为 $(p, b, c)$ 的群元素。
///
/// # 参数
/// - `discriminant`: 判别式 $\Delta$ (负数)
/// - `count`: 需要生成的扰动数量
pub fn generate_perturbations(discriminant: &BigInt, count: usize) -> Vec<ClassGroupElement> {
    let mut perturbations = Vec::with_capacity(count);
    // 从最小的素数开始搜索
    let mut p_candidate = 2u64;

    while perturbations.len() < count {
        if is_prime(p_candidate) {
            // 尝试为当前素数 p 找到合法的 b
            // 这隐含了检查 Legendre/Kronecker 符号
            if let Some(element) = try_create_prime_form(discriminant, p_candidate) {
                perturbations.push(element);
            }
        }
        // 简单的递增搜索，虽然不是最高效的素数生成方式，但对于寻找前几十个扰动足够快
        p_candidate += 1;
    }

    perturbations
}

/// 尝试为素数 p 构造一个类群元素 (p, b, c)
/// 如果 p 不分裂（即无法找到满足条件的 b），返回 None。
fn try_create_prime_form(discriminant: &BigInt, p: u64) -> Option<ClassGroupElement> {
    let p_bi = BigInt::from(p);
    let four_p = BigInt::from(4) * &p_bi;
    
    // 我们需要解方程: b^2 ≡ Delta (mod 4p)
    // 且 b 的奇偶性通常与 Delta 相同（如果是虚二次域，Delta通常是奇数，所以b也是奇数）
    
    // 计算目标余数: target = Delta mod 4p
    // 注意：BigInt 的 % 可能会返回负数，我们需要正余数
    let target = discriminant.mod_floor(&four_p);

    // 暴力搜索 b ∈ [0, 4p)
    // 优化：b 的奇偶性通常与 Delta 相同
    // 大多数情况下 Delta = 1 mod 4 (即 -M, M=3 mod 4)，所以 Delta 是奇数
    let start = if discriminant.is_odd() { 1 } else { 0 };
    let step = 2;
    
    // 这里的 limit 可以优化，但 4p 对于小素数来说非常小，暴力法足够安全且无 bug
    let limit = 4 * p; 
    let mut b_curr = start;

    while b_curr < limit {
        let b_bi = BigInt::from(b_curr);
        let b_sq = &b_bi * &b_bi;

        // 检查 b^2 ≡ Delta (mod 4p)
        if b_sq.mod_floor(&four_p) == target {
            // 找到了合法的 b！
            // 计算 c = (b^2 - Delta) / 4p
            let numerator = &b_sq - discriminant;
            let c_val = numerator / &four_p;

            // 构造元素
            // 注意：这里我们假设 ClassGroupElement 结构体字段是公开的。
            // 如果不是，需要使用对应的构造函数。
            return Some(ClassGroupElement {
                a: p_bi,
                b: b_bi,
                c: c_val,
            });
        }
        b_curr += step;
    }

    None
}

/// 基础的素数判定
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_perturbation_generation() {
        // 测试用例：Delta = -23
        // 对应的类群很小，只有 h(-23)=3。
        // 分裂的素数：
        // (-23/2) = 1 (因为 -23 = 1 mod 8) -> p=2 分裂
        // (-23/3) = 1 (因为 -23 = 1 mod 3) -> p=3 分裂
        let delta = BigInt::from(-23);
        let perts = generate_perturbations(&delta, 2);

        assert_eq!(perts.len(), 2);
        
        // 检查第一个元素 (p=2)
        assert_eq!(perts[0].a, BigInt::from(2));
        // b^2 = -23 mod 8 -> b^2 = 1 mod 8. b=1, 3, 5, 7 都可以。
        // 我们的算法应该找到最小的正解。
        
        // 检查第二个元素 (p=3)
        assert_eq!(perts[1].a, BigInt::from(3));
    }
}
