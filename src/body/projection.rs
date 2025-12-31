use crate::soul::algebra::ClassGroupElement;

/// FNV-1a 64-bit constants
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// 实现“阿廷投影” (Artin-like Projection)
///
/// 该函数将一个代数结构（理想类群元素）确定性地投影到一个有限域 Z_p 上。
/// 在 v-PuNN 模型中，这代表了从“潜意识的代数状态”到“显意识的决策符号”的坍缩过程。
///
/// # 参数
/// * `g` - 理想类群元素 (ClassGroupElement)，通常包含系数 a, b, c
/// * `p` - 投影的模数 (素数基底)，决定了输出空间的大小
///
/// # 返回
/// * `u64` - 在 [0, p-1] 范围内的投影值
pub fn project_to_digit(g: &ClassGroupElement, p: u64) -> u64 {
    // 1. 初始化哈希状态 (FNV-1a 算法)
    let mut hash = FNV_OFFSET_BASIS;

    // 2. 提取并混合系数
    // 注意：为了保证通用性且不依赖具体的 BigInt 实现细节，
    // 我们这里使用 Debug 或 Display 的字节表示来作为哈希源。
    // 在生产环境中，直接操作二进制位会更高效。
    // 假设 g 包含 (a, b, c)，这些系数唯一确定了一个群元素。
    
    // 混合系数 a, b, c (通过字符串表示，确保确定性)
    // 这种方式不仅捕获了数值，还捕获了结构。
    let raw_repr = format!("{:?}", g); 

    for byte in raw_repr.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    // 3. 额外的雪崩效应 (Avalanche Mixer)
    // 确保输入的微小变化（如 a 变动 1）能导致输出的剧烈变化
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51afd7ed558ccd);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xc4ceb9fe1a85ec53);
    hash ^= hash >> 33;

    // 4. 投影到有限域 Z_p
    hash % p
}

#[cfg(test)]
mod tests {
    use super::*;
    // 这里的测试依赖于 Mock 的 ClassGroupElement，
    // 在实际集成时需要确保 soul::algebra 模块可用。
}
