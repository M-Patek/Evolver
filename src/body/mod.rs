// Body 模块定义
// 负责将代数状态实体化为逻辑动作

pub mod adapter;
pub mod projection;

// 之前的 navigator, topology, decoder 已被集成到 projection 中，在此移除导出
