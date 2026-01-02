/// 意志轨迹记录者 (Will Tracer)
/// 定义了优化器在搜索过程中如何记录其决策路径的接口。
///
/// 通过将记录行为抽象为 Trait，我们实现了：
/// 1. 原生模式 (Silent): 零开销，不分配内存记录轨迹。
/// 2. 证明模式 (Proven): 记录每一步的生成元索引，用于构建 ProofBundle。
pub trait WillTracer {
    /// 当优化器接受一个新的状态转移时调用
    /// generator_idx: 被选择的生成元索引
    fn on_accept(&mut self, generator_idx: usize);

    /// 当优化器拒绝一个状态转移时调用
    fn on_reject(&mut self);
}

/// 静默记录者 (SilentTracer)
/// 用于 "原生输出模式"，完全忽略轨迹记录，实现零开销。
pub struct SilentTracer;

impl WillTracer for SilentTracer {
    #[inline(always)]
    fn on_accept(&mut self, _generator_idx: usize) {
        // Do nothing (The Void stares back)
    }

    #[inline(always)]
    fn on_reject(&mut self) {
        // Do nothing
    }
}

/// 证明记录者 (ProvenTracer)
/// 用于 "可验证模式"，忠实记录每一步的决策，用于构建 ProofBundle。
pub struct ProvenTracer {
    pub trace: Vec<usize>,
}

impl ProvenTracer {
    pub fn new() -> Self {
        Self { trace: Vec::new() }
    }
}

impl Default for ProvenTracer {
    fn default() -> Self {
        Self::new()
    }
}

impl WillTracer for ProvenTracer {
    fn on_accept(&mut self, generator_idx: usize) {
        self.trace.push(generator_idx);
    }

    fn on_reject(&mut self) {
        // 目前 ProofBundle 不需要记录拒绝的步骤，
        // 因为验证只需重放成功的路径。
        // 但如果有基于时间的侧信道防御需求，未来可能需要记录。
    }
}
