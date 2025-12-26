// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

#[cfg(test)]
mod tests {
    use crate::phase3::core::algebra::ClassGroupElement;
    use crate::phase3::core::affine::AffineTuple;
    use rug::Integer;

    fn setup_env() -> Integer {
        // 使用测试判别式 (Small prime for speed)
        // M = 1000003 (3 mod 4) -> Delta = -M = 1 mod 4
        let m = Integer::from(1000003); 
        let discriminant = -m;
        discriminant
    }

    /// 🌊 [CRITICAL TEST]: 验证流式演化的状态恒定性
    /// 证明系统可以处理无限长度的序列而不会发生内存/位宽爆炸
    /// 
    /// 理论基础：S_new = S_old^p * q
    /// 在这一步中，p 被作为指数立即消耗，只有结果状态 S_new 被保留。
    #[test]
    fn test_state_streaming_constant_size() {
        let discriminant = setup_env();
        let mut state = ClassGroupElement::identity(&discriminant);
        
        println!("🌊 [Test] Starting State Streaming Evolution...");
        
        // 记录初始状态大小
        let initial_bits = state.a.significant_bits();
        println!("   Initial State Size: {} bits", initial_bits);

        // 模拟 100 步演化 (如果是旧的累积模式，P因子早已爆炸)
        for i in 0..100 {
            // 模拟输入 Token (P) 和 移位 (Q)
            let p = Integer::from(1009); 
            let q = ClassGroupElement::generator(&discriminant); 
            
            // Apply: S_new = S_old^p * q
            // 关键点：这里 p 被立即消耗掉了，state 的大小应当回弹到类群元素的标准大小
            state = state.apply_affine(&p, &q, &discriminant).unwrap();
            
            if i % 20 == 0 {
                let size = state.a.significant_bits();
                println!("   Step {}: State Size = {} bits", i, size);
                
                // 断言：状态大小受判别式约束，不随时间线性增长
                // 允许一定的波动 (reduction 后的正常浮动)，但绝不能持续增长
                assert!(size < discriminant.significant_bits() + 200, "State explosion detected!");
            }
        }
        println!("✅ State Streaming test passed. No explosion detected.");
    }

    /// 💥 [BOUNDARY TEST]: 验证 P-Factor 熔断机制
    /// 试图进行超出 MAX_CHUNK_P_BITS 的累积，应触发 Panic 或 Err
    /// 
    /// 证伪性：这证明了系统拒绝将“无限”压缩为“有限”的尝试。
    #[test]
    #[should_panic(expected = "Falsified")] // 预期会捕获到包含 "Falsified" 的错误信息
    fn test_legacy_accumulation_fuse() {
        let discriminant = setup_env();
        let mut accumulator = AffineTuple::identity(&discriminant);
        
        println!("💥 [Test] Testing Legacy Accumulation Fuse...");

        // 模拟恶意攻击者试图构造一个巨大的 P 因子
        // 每次 P 增加 ~10 bits，循环 1000 次将达到 10000 bits > 8192
        for _ in 0..1000 {
            let p = Integer::from(1009); 
            let q = ClassGroupElement::identity(&discriminant);
            let op = AffineTuple { p_factor: p, q_shift: q };
            
            // 这里应当在某一次循环中触发 Err/Panic
            // 因为 compose 内部有硬性的位宽检查
            accumulator = accumulator.compose(&op, &discriminant).unwrap();
        }
    }
}
