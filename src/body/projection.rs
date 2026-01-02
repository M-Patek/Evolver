use crate::soul::algebra::IdealClass;
use sha2::{Digest, Sha256};

/// 投影仪 (Projector)
/// 负责将抽象的代数状态（Soul）“显化”为可观测的特征（Body）。
///
/// 这里实现了“双重投影架构” (Dual Projection Architecture)：
/// 1. 连续投影 (Continuous): 用于优化器的启发式引导 (Lipschitz 连续)。
/// 2. 精确投影 (Exact): 用于生成最终的逻辑路径 (离散且混沌)。
#[derive(Debug, Clone)]
pub struct Projector {
    seed_discriminator: u64,
}

impl Projector {
    pub fn new(seed_discriminator: u64) -> Self {
        Self { seed_discriminator }
    }

    /// 连续投影 (Psi_topo): S -> R^n
    /// 将四元数状态映射到连续的特征向量上。
    ///
    /// [数学原理]
    /// 我们将四元数 q = a + bi + cj + dk 视为 R^4 空间中的向量。
    /// 为了消除幅度（路径长度）的影响，仅保留方向信息，我们将其投影到单位超球面 S^3 上。
    /// v = q / |q|
    /// 这样的映射满足 Lipschitz 连续性：状态的微小旋转导致特征的微小变化。
    pub fn project_continuous(&self, state: &IdealClass) -> Vec<f64> {
        let q = state.value;
        
        // 转换为浮点数
        let vecRaw = vec![
            q.a as f64,
            q.b as f64,
            q.c as f64,
            q.d as f64,
        ];

        // 计算欧几里得范数 (L2 Norm)
        let norm_sq: f64 = vecRaw.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();

        // 归一化投影 (避免除以零)
        if norm < 1e-9 {
            return vec![0.0; 4];
        }

        vecRaw.iter().map(|x| x / norm).collect()
    }

    /// 精确投影 (Psi_exact): S -> Z_p
    /// 将四元数状态坍缩为一个确定性的、混沌的离散值。
    /// 用于生成 ProofAction 或验证 Hash。
    ///
    /// [因果敏感性]
    /// 由于 state.value 是路径上所有算子的有序乘积，
    /// 这里的哈希值实际上是对整个因果链的数字签名。
    pub fn project_exact(&self, state: &IdealClass, time_step: u64) -> u64 {
        let mut hasher = Sha256::new();
        
        // 输入系统参数
        hasher.update(self.seed_discriminator.to_be_bytes());
        
        // 输入时间步 (区分同一状态在不同时刻的观测)
        hasher.update(time_step.to_be_bytes());
        
        // 输入四元数完整状态 (a, b, c, d)
        // 这里的微小差异会导致输出的雪崩效应
        hasher.update(state.value.a.to_be_bytes());
        hasher.update(state.value.b.to_be_bytes());
        hasher.update(state.value.c.to_be_bytes());
        hasher.update(state.value.d.to_be_bytes());

        let result = hasher.finalize();

        // 取前8个字节作为 u64 输出
        u64::from_be_bytes(result[0..8].try_into().unwrap_or([0; 8]))
    }
}
