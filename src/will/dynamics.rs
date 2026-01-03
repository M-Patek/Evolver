// Copyright (c) 2025 M-Patek
// Part of the Evolver Project
//
// "When the path curves inward, walk; when it curves outward, beam."

use nalgebra::{DMatrix, DVector};
use std::cmp::Ordering;

/// 优化模式 (Optimization Mode)
/// 决定了 Will 如何在代数流形上移动。
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OptimizationMode {
    /// 梯度流模式 (Gradient Flow)
    /// 适用于正曲率 ($\kappa > 0$) 或平坦区域。
    /// 在这些区域，局部最优方向通常指向全局最优，因此采用贪婪策略。
    GradientFlow,

    /// 双曲波束模式 (Hyperbolic Beam)
    /// 适用于负曲率 ($\kappa < 0$) 区域。
    /// 在这些区域，空间呈指数级发散（树状结构），贪婪搜索极易陷入局部死胡同。
    /// 必须同时保留多个候选路径 (Beam Width)，模拟波的扩散。
    HyperbolicBeam,
}

/// Sinkhorn 引擎 (Sinkhorn Engine)
/// 
/// 用于高效计算两个概率分布之间的熵正则化最优传输距离 (Sinkhorn Divergence)。
/// 相比于传统的 Wasserstein Distance (EMD)，Sinkhorn 算法是可微的，且计算复杂度更低。
pub struct SinkhornEngine {
    /// 熵正则化系数 (Entropic Regularization epsilon)
    /// 较大的 epsilon 会使传输计划更加模糊（趋向于均匀分布），计算收敛更快。
    /// 较小的 epsilon 会逼近真实的 OT 距离，但数值稳定性变差。
    reg_epsilon: f64,
    
    /// 最大迭代次数 (Sinkhorn-Knopp iterations)
    max_iter: usize,
}

impl SinkhornEngine {
    /// 初始化 Sinkhorn 引擎
    /// reg: 正则化系数，通常取 0.1 ~ 1.0 之间
    pub fn new(reg: f64) -> Self {
        Self { 
            reg_epsilon: reg, 
            max_iter: 50 // 对于近似计算，通常 20-50 次迭代足矣
        }
    }

    /// 计算 Sinkhorn Divergence
    /// 
    /// $$ W_\epsilon(a, b) = \langle C, P^* \rangle - \epsilon H(P^*) $$
    /// 
    /// # 参数
    /// * `a`, `b`: 两个离散概率分布向量 (Sum must be 1.0)
    /// * `C`: 成本矩阵 (Cost Matrix)，C_ij 表示从 i 移动到 j 的代价
    pub fn compute_divergence(&self, a: &DVector<f64>, b: &DVector<f64>, C: &DMatrix<f64>) -> f64 {
        let n = a.len();
        
        // 1. 计算吉布斯核 (Gibbs Kernel) K = exp(-C / epsilon)
        // 这是一个逐元素操作
        let K = C.map(|x| (-x / self.reg_epsilon).exp());
        
        // 初始化缩放向量 u, v
        let mut u = DVector::from_element(n, 1.0 / n as f64);
        let mut v = DVector::from_element(n, 1.0 / n as f64);

        // 2. Sinkhorn-Knopp 迭代 (Matrix Scaling)
        // 目标是找到对角矩阵 D1(u), D2(v) 使得 P = D1 * K * D2 满足边缘分布约束
        for _ in 0..self.max_iter {
            // v = b ./ (K^T * u)
            let Kt_u = K.transpose() * &u;
            // 数值稳定保护：避免除以零
            v = b.zip_map(&Kt_u, |num, den| if den < 1e-9 { 0.0 } else { num / den });

            // u = a ./ (K * v)
            let K_v = &K * &v;
            u = a.zip_map(&K_v, |num, den| if den < 1e-9 { 0.0 } else { num / den });
        }

        // 3. 计算传输成本
        // Transport Cost = <u, (K .* C) v>
        // 注意：Sinkhorn Distance 实际上是 <P, C>
        // P_ij = u_i * K_ij * v_j
        // sum(P_ij * C_ij) = sum(u_i * K_ij * v_j * C_ij)
        
        let K_dot_C = K.component_mul(C); // 逐元素乘法
        let right_term = &K_dot_C * &v;   // 矩阵向量乘法
        let distance = u.dot(&right_term);
        
        distance
    }
}

/// 动态优化器 (Dynamic Optimizer)
/// 
/// 管理搜索策略的状态机。根据当前的几何曲率 ($\kappa$) 动态切换
/// "贪婪爬山" (Gradient Flow) 和 "波束搜索" (Hyperbolic Beam)。
pub struct DynamicOptimizer {
    /// 当前优化模式
    pub mode: OptimizationMode,
    
    /// 曲率阈值
    /// 当 $\kappa$ 低于此值时，认为进入负曲率陷阱。
    curvature_threshold: f64, 
    
    /// 波束宽度 (Beam Width)
    /// 在 HyperbolicBeam 模式下保留的候选路径数量。
    beam_width: usize,
}

impl DynamicOptimizer {
    pub fn new() -> Self {
        Self {
            mode: OptimizationMode::GradientFlow,
            curvature_threshold: -0.5, // 经验值：低于 -0.5 说明发散严重
            beam_width: 5,             // 保持 5 条平行宇宙
        }
    }

    /// 核心状态机：根据 Ollivier-Ricci 曲率决定优化策略
    /// 
    /// # 参数
    /// * `kappa`: 当前边的离散 Ricci 曲率
    pub fn switch_mode(&mut self, kappa: f64) -> OptimizationMode {
        if kappa < self.curvature_threshold {
            // 进入负曲率区域 -> 开启波束搜索
            if self.mode != OptimizationMode::HyperbolicBeam {
                // println!("[Dynamics] WARN: High Negative Curvature ({:.2}). Switching to BEAM SEARCH.", kappa);
                self.mode = OptimizationMode::HyperbolicBeam;
            }
        } else {
            // 回到平坦或正曲率区域 -> 回归梯度流
            if self.mode != OptimizationMode::GradientFlow {
                // println!("[Dynamics] INFO: Curvature Stabilized ({:.2}). Returning to GRADIENT FLOW.", kappa);
                self.mode = OptimizationMode::GradientFlow;
            }
        }
        self.mode
    }

    /// 执行一步优化：根据当前模式筛选候选状态
    /// 
    /// # 参数
    /// * `candidates`: 下一步可能的候选状态集合 (Hash or ID)
    /// * `objective_fn`: 计算状态能量/成本的闭包函数
    /// 
    /// # 返回
    /// * `Vec<u64>`: 筛选后保留的下一代状态
    pub fn step<F>(&self, candidates: Vec<u64>, objective_fn: F) -> Vec<u64> 
    where F: Fn(u64) -> f64 
    {
        if candidates.is_empty() {
            return vec![];
        }

        match self.mode {
            OptimizationMode::GradientFlow => {
                // 梯度模式：贪婪选择最好的一个 (Greedy Best-First)
                // 模拟水流沿最陡峭路径下山
                if let Some(best) = candidates.into_iter()
                    .min_by(|a, b| {
                        let cost_a = objective_fn(*a);
                        let cost_b = objective_fn(*b);
                        cost_a.partial_cmp(&cost_b).unwrap_or(Ordering::Equal)
                    }) 
                {
                    vec![best]
                } else {
                    vec![]
                }
            },
            OptimizationMode::HyperbolicBeam => {
                // 波束模式：保留 top-k
                // 模拟波在双曲空间的发散，避免过早收敛到错误的局部极小值
                let mut scored: Vec<(u64, f64)> = candidates.into_iter()
                    .map(|s| (s, objective_fn(s)))
                    .collect();
                
                // 排序：从小到大（能量越低越好）
                scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
                
                // 取前 beam_width 个
                scored.into_iter()
                    .take(self.beam_width)
                    .map(|(s, _)| s)
                    .collect()
            }
        }
    }
}
