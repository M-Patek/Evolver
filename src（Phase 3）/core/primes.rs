// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::Integer;
use blake3::Hasher;

/// 🛡️ Hash-to-Prime Map (With XOF Full Entropy)
/// 将任意字符串确定性地映射为一个大素数。
/// 
/// [SECURITY UPDATE]: 启用了 BLAKE3 XOF，确保生成的素数在整个 `bit_size` 空间内均匀分布。
pub fn hash_to_prime(user_id: &str, bit_size: u32) -> Result<Integer, String> {
    let mut nonce = 0u64;
    let optimal_search_limit = 1000; 
    let num_bytes = ((bit_size + 7) / 8) as usize;
    
    // --- Phase 1: 概率性哈希试探 (XOF Enabled) ---
    while nonce < optimal_search_limit {
        let mut hasher = Hasher::new();
        hasher.update(&(user_id.len() as u64).to_le_bytes());
        hasher.update(user_id.as_bytes());
        hasher.update(&nonce.to_le_bytes());
        
        // [CRITICAL FIX]: 使用 XOF 填满整个缓冲区
        let mut entropy_buffer = vec![0u8; num_bytes];
        let mut output_reader = hasher.finalize_xof();
        output_reader.fill(&mut entropy_buffer);

        let mut candidate = Integer::from_digits(&entropy_buffer, rug::integer::Order::Lsf);
        
        // 强制设置最高位和最低位
        candidate.set_bit(bit_size - 1, true);
        candidate.set_bit(0, true);

        // 小素数筛
        if candidate.mod_u(3) == 0 || candidate.mod_u(5) == 0 {
            nonce += 1;
            continue;
        }

        // Miller-Rabin
        if candidate.is_probably_prime(25) != rug::integer::IsPrime::No {
            return Ok(candidate);
        }

        nonce += 1;
    }
    
    // --- Phase 2: 确定性保底扫描 (Fallback) ---
    // 即使在 Fallback 模式下，我们也应该基于一个高熵的起始点
    
    let mut hasher = Hasher::new();
    hasher.update(b"HTP_PRIME_FALLBACK_V1::");
    hasher.update(user_id.as_bytes());
    
    let mut entropy_buffer = vec![0u8; num_bytes];
    let mut output_reader = hasher.finalize_xof();
    output_reader.fill(&mut entropy_buffer);
    
    let mut fallback_candidate = Integer::from_digits(&entropy_buffer, rug::integer::Order::Lsf);
    fallback_candidate.set_bit(bit_size - 1, true);
    fallback_candidate.set_bit(0, true);

    fallback_candidate.next_prime_mut();

    Ok(fallback_candidate)
}
