use rug::{Float, ops::Pow};

/// 评估器配置
/// 定义了 VAPO 对能量感知的精度要求
pub struct EvaluatorConfig {
    /// 计算精度 (bits)。
    /// 建议值：512 (对于 D=60 左右的深度)。
    /// 如果路径深度超过 100，建议提升至 1024。
    pub precision: u32,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self { precision: 512 }
    }
}

/// 高精度评估器 (High Precision Evaluator)
/// 
/// 负责计算当前状态与目标状态之间的残差 (Residual)。
/// 
/// [AUDIT FIX]: 之前的版本使用 f64，在路径范数 N > 10^45 时会导致
/// 机器精度溢出 (1.0 + 1e-180 == 1.0)，引发梯度死亡。
/// 现在使用 MPFR Float (rug crate) 进行任意精度计算。
pub struct HighPrecisionEvaluator {
    precision: u32,
}

impl HighPrecisionEvaluator {
    /// 创建一个新的评估器
    pub fn new(config: EvaluatorConfig) -> Self {
        Self {
            precision: config.precision,
        }
    }

    /// 创建一个零能量目标 (Perfect State)
    pub fn zero_energy(&self) -> Float {
        Float::with_val(self.precision, 0.0)
    }

    /// 计算高精度残差
    /// 
    /// Formula: Residual = |Target - Current|
    /// 
    /// 这里的关键是：即使差异只有 1e-180，512-bit 的 Float 也能精确捕捉到，
    /// 从而为 VAPO 提供有效的下降梯度。
    pub fn calculate_residual(&self, target: &Float, current: &Float) -> Float {
        // 使用 rug 的高精度绝对值计算
        // 拒绝 f64 的截断误差！
        (target - current).abs()
    }

    /// 检查是否收敛
    /// 
    /// 判断残差是否小于某个极小的阈值 (epsilon)。
    /// 注意：这里的 epsilon 也必须是高精度的。
    pub fn has_converged(&self, residual: &Float, epsilon_exponent: i32) -> bool {
        // 构造高精度阈值 10^exponent
        let base = Float::with_val(self.precision, 10.0);
        let threshold = base.pow(epsilon_exponent);
        
        residual < &threshold
    }

    /// 获取当前的精度设置
    pub fn precision(&self) -> u32 {
        self.precision
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_death_fix() {
        let precision = 512;
        let evaluator = HighPrecisionEvaluator::new(EvaluatorConfig { precision });

        // 模拟梯度死亡场景：
        // Base = 10^180
        // Delta = 1 (微扰)
        let base_val = Float::with_val(precision, 10.0).pow(180);
        let perturbed_val = base_val.clone() + Float::with_val(precision, 1.0);

        // 在 f64 中，(10^180 + 1) - 10^180 == 0.0 (精度丢失)
        // 在 rug::Float 中，应该能找回这个 1.0
        
        let diff = perturbed_val - base_val;
        let expected = Float::with_val(precision, 1.0);
        
        // 验证我们找回了丢失的精度
        assert!((diff - expected).abs() < Float::with_val(precision, 1e-10));
        println!("喵！成功在 10^180 的大数海洋中捕捉到了 1.0 的微扰！");
    }
}
