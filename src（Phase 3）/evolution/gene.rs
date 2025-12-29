// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::Integer;
use crate::phase3::core::affine::AffineTuple;
use std::cmp::Ordering;

/// 🧬 ProbeGene: 探针基因
/// 代表一个在代数空间中探索的“个体”。
/// 它携带了到达当前位置的完整逻辑链条 (Logic DNA) 和微调参数 (Control DNA)。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeGene {
    /// [Logic DNA]: 核心语义素数 (P_weight)
    /// 决定了逻辑的“大方向” (因果、转折、递进...)
    pub p_weight: Integer,

    /// [Control DNA]: 线性偏差向量 (Bias Vector)
    /// 决定了逻辑的“微调” (基于 Theorem 5.7)
    /// 这里简化为 usize 向量，实际应用中可能需要更复杂的结构
    pub bias_vector: Vec<usize>,

    /// [Lineage]: 族谱/深度
    /// 记录这个探针存活了多少代 (Depth)
    pub depth: usize,

    /// [Memory]: 累积的代数状态
    /// 用于断点续传，避免从头计算 (Checkpoint)
    pub current_state: AffineTuple,
}

/// 📊 ProbeState: 用于优先队列的包装器
/// 实现了 Ord trait，以便在“分形网搜索”中进行排序。
/// 排序逻辑：适应度越高，优先级越高。
#[derive(Clone, Debug)]
pub struct ProbeState {
    pub gene: ProbeGene,
    
    /// [Fitness]: 适应度分数
    /// 由 存活深度(Depth) + 逻辑自洽性(Consistency) + 预测概率(Prob) 组成
    pub fitness_score: f64,
}

// 实现大根堆排序：适应度高的排前面
impl Ord for ProbeState {
    fn cmp(&self, other: &Self) -> Ordering {
        // f64 不实现 Ord，所以我们需要 partial_cmp 并处理 NaN
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for ProbeState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.fitness_score.partial_cmp(&other.fitness_score)
    }
}

impl PartialEq for ProbeState {
    fn eq(&self, other: &Self) -> bool {
        self.fitness_score == other.fitness_score
    }
}

impl Eq for ProbeState {}
