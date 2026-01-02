//! The Will Module (意志模块)
//! 
//! 负责系统的意图导向和优化搜索。
//! 在本体论修正案中，Will 变成了在定四元数算术格（Pizer Graph）上的导航员。
//! 
//! [v2.3 Update] 引入 Ricci 流 (ricci.rs) 以解决负曲率死锁问题。

pub mod evaluator;
pub mod optimizer;
pub mod perturber;
pub mod tracer;
pub mod ricci; // [New] 注册 Ricci 流模块
