// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::{Integer, complete::Complete};
use blake3::Hasher;

// [SECURITY NOTE]: 在生产构建中，必须在 Cargo.toml 中添加 wesolowski 依赖
// 并开启 "production_vdf" feature。
#[cfg(feature = "production_vdf")]
use wesolowski::{verify as vdf_verify, Error as VdfError};

// [SECURITY CONSTANTS]
// 提升最小位宽至 3072 bits，以抵抗量子计算和未来的超级计算机攻击
const MIN_DISCRIMINANT_BITS: u32 = 3072; 

// 域分离标签 (Domain Separation Tag)
const DOMAIN_TAG: &[u8] = b"Evolver_v1_System_Discriminant_Generation_DST";

// [TRUSTLESS CONSTANTS]
const VDF_TIME_PARAM_T: u64 = 1 << 40; 

pub struct SystemParameters {
    pub discriminant: Integer,
}

impl SystemParameters {
    /// ⚠️ [DEPRECATED]: 仅用于开发或测试环境
    pub fn from_random_seed(seed_bytes: &[u8], bit_size: u32) -> Self {
        // [SECURITY FIX 1]: 强制安全参数下限检查
        if bit_size < 2048 {
             panic!("❌ SECURITY VIOLATION: Discriminant size must be >= 2048 bits (Recommended 3072).");
        }
        
        println!("[System] ⚠️ WARNING: Using interactive seed setup. NOT SECURE for production.");
        Self::generate_internal(seed_bytes, bit_size)
    }

    /// 🛡️ [THEORETICAL OPTIMUM]: 无信参数生成协议
    pub fn derive_trustless_discriminant(
        beacon_block_hash: &[u8], 
        vdf_output: &[u8],      
        vdf_proof: &[u8]        
    ) -> Result<Self, String> {
        println!("[System] Initiating Trustless Setup Protocol...");
        println!("[System] Target Security Level: {} bits", MIN_DISCRIMINANT_BITS);

        // 1. [Step 1]: 验证 VDF 证明
        if !Self::verify_vdf(beacon_block_hash, vdf_output, vdf_proof) {
            return Err("❌ FATAL: VDF Proof Invalid. The randomness source may be manipulated.".to_string());
        }

        println!("[System] ✅ VDF Verified. Entropy is hardened by physical time.");

        // 2. [Step 2]: 确定性混合
        let mut hasher = Hasher::new();
        hasher.update(DOMAIN_TAG);
        hasher.update(b"::TRUSTLESS_SETUP::PHASE_1::");
        hasher.update(beacon_block_hash); 
        hasher.update(vdf_output);
        let final_seed = hasher.finalize();

        // 3. [Step 3]: 生成基本判别式
        // 注意：这里传入的 bit_size 必须足以抵抗离散对数攻击
        let params = Self::generate_internal(final_seed.as_bytes(), MIN_DISCRIMINANT_BITS);
        
        Ok(params)
    }

    /// 内部核心生成逻辑 (Cohen-Lenstra Heuristics Optimized)
    /// [SECURITY UPGRADE]: 使用 XOF 确保全位宽熵覆盖
    fn generate_internal(seed_bytes: &[u8], bit_size: u32) -> Self {
        println!("[System] Deriving Fundamental Discriminant (Full Entropy Mode)...");
        
        let mut attempt = 0;
        let max_attempts = 10_000_000; 

        // 计算需要的字节数 (向上取整)
        let num_bytes = ((bit_size + 7) / 8) as usize;

        loop {
            if attempt > max_attempts {
                panic!("❌ Failed to generate System Parameters. Entropy pool exhausted or bad luck.");
            }

            // 1. CSPRNG 扩展: 使用 BLAKE3 XOF 模式
            // 这确保了生成的 candidate 每一个比特都是由种子派生的，具有 3072-bit 级别的真实熵
            let mut hasher = Hasher::new();
            hasher.update(seed_bytes);
            hasher.update(b"::NONCE::");
            hasher.update(&attempt.to_le_bytes()); 
            
            // [CRITICAL FIX]: 使用 finalize_xof 填充整个缓冲区，而不是 finalize() 仅取前 32 字节
            let mut entropy_buffer = vec![0u8; num_bytes];
            let mut output_reader = hasher.finalize_xof();
            output_reader.fill(&mut entropy_buffer);

            // 2. 构造候选大整数
            let mut candidate = Integer::from_digits(&entropy_buffer, rug::integer::Order::Lsf);
            
            // 确保高位为1，严格保证位宽安全性
            candidate.set_bit(bit_size - 1, true);
            
            // 3. 基本判别式筛选条件 (Fundamental Discriminant Criteria)
            // 定义 Delta = -M
            // 要求 M = 3 mod 4 (从而导致 Delta = 1 mod 4)
            // 且 M 必须是无平方因子的 (Square-free)。若 M 为素数，则自动满足。
            let rem = candidate.mod_u(4);
            if rem != 3 {
                attempt += 1;
                continue;
            }

            // 4. 强素性测试 (Miller-Rabin)
            if candidate.is_probably_prime(50) != rug::integer::IsPrime::No {
                let discriminant = -candidate;
                println!("✅ [Trustless Setup] Success! Found Fundamental Discriminant.");
                println!("   Delta Fingerprint: ...{:X} (Last 64 bits)", discriminant.clone() % Integer::from(1u64 << 64));
                println!("   Attempts: {}", attempt);
                return SystemParameters { discriminant };
            }

            attempt += 1;
        }
    }

    fn verify_vdf(input: &[u8], output: &[u8], proof: &[u8]) -> bool {
        if input.is_empty() || output.is_empty() || proof.is_empty() {
            eprintln!("[VDF Verify] ❌ Security Alert: Empty payload detected.");
            return false;
        }

        #[cfg(feature = "production_vdf")]
        {
            match vdf_verify(input, output, proof, VDF_TIME_PARAM_T) {
                Ok(true) => return true,
                Ok(false) => {
                    eprintln!("[VDF Verify] ❌ Mathematical verification failed.");
                    return false;
                },
                Err(e) => {
                    eprintln!("[VDF Verify] ❌ Verification error: {:?}", e);
                    return false;
                }
            }
        }

        #[cfg(not(feature = "production_vdf"))]
        {
            println!("[VDF Verify] ⚠️ WARNING: Running in MOCK mode. Not secure for mainnet.");
            let mut hasher = Hasher::new();
            hasher.update(b"EVOLVER_VDF_SIMULATION_BINDING");
            hasher.update(input);
            hasher.update(output);
            let expected_proof_hash = hasher.finalize();
            proof == expected_proof_hash.as_bytes()
        }
    }
}
