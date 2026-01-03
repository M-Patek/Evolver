// Copyright (c) 2025 M-Patek
// Part of the Evolver Project
//
// "The Grand Loop: Where Soul, Will, and Body dance in unison."

use std::collections::HashMap;
use nalgebra::DVector;

use crate::soul::governor::SpectralGovernor;
use crate::soul::lifter::StateLifter;
use crate::soul::algebra::AlgebraicState;
use crate::will::dynamics::{DynamicOptimizer, OptimizationMode};
use crate::body::guard_proxy::LazyGuard;
use crate::dsl::stp_bridge::LogicEvaluator;

/// Evolver 引擎 (Evolver Engine)
/// 
/// 系统的核心控制器，实现了 "Grand Loop" 架构。
/// 它协调各个子系统 (Governor, Lifter, Optimizer, Guard) 协同工作，
/// 驱动代数状态向逻辑真理演化。
pub struct EvolverEngine {
    /// 当前的代数状态 (The Soul)
    state: AlgebraicState,
    
    /// 谱隙守护者：监控代数空间的连通性健康度
    governor: SpectralGovernor,
    
    /// 状态提升器：负责跨代数空间的记忆移植
    lifter: StateLifter,
    
    /// 动态优化器：负责在流形上规划路径 (The Will)
    optimizer: DynamicOptimizer,
    
    /// 惰性卫士：负责审查解的拓扑合法性 (The Body's Immune System)
    guard: LazyGuard,
    
    /// 逻辑评估器：计算几何损失和验证最终逻辑
    evaluator: LogicEvaluator,
    
    /// 运行时计数器
    epoch: usize,
}

impl EvolverEngine {
    /// 初始化引擎
    /// initial_p: 初始代数空间的素数参数
    pub fn new(initial_p: u64) -> Self {
        Self {
            state: AlgebraicState::new_root(initial_p),
            governor: SpectralGovernor::new(initial_p),
            lifter: StateLifter::new(),
            optimizer: DynamicOptimizer::new(),
            guard: LazyGuard::new(),
            evaluator: LogicEvaluator::new(),
            epoch: 0,
        }
    }

    /// THE GRAND LOOP: 意志的主循环
    /// 
    /// 这是系统的主线程，永不停止，直到找到真理或熵耗尽。
    /// 
    /// # 返回
    /// * `Ok(AlgebraicState)`: 找到并通过验证的真理状态
    /// * `Err(String)`: 熵耗尽或发生不可恢复错误
    pub fn evolve(&mut self) -> Result<AlgebraicState, String> {
        println!("[System] EVOLVER ENGINE IGNITION. Target: Absolute Logic.");

        loop {
            self.epoch += 1;
            
            // 熵耗尽保护 (Entropy Exhaustion)
            if self.epoch > 100_000 { 
                return Err("Entropy Exhausted: Maximum epochs reached without convergence.".to_string()); 
            }

            // =============================================================
            // Phase 1: 宇宙常数检查 (Spectral Governance)
            // =============================================================
            // 这是一个昂贵的操作，我们仅在固定间隔检查，或者当系统明显停滞时检查。
            if self.epoch % 50 == 0 {
                // 探索局部图结构：获取节点集和邻接关系
                let (nodes, adj) = self.state.explore_local_graph(30);
                
                if !self.governor.check_spectral_gap(&nodes, &adj) {
                    // [CRITICAL] 谱隙关闭，代数空间已死 (Spectral Collapse)。
                    // 局部图结构变成了 "细管" 或 "哑铃"，随机游走效率极低。
                    println!("[System] Spectral Collapse detected at Epoch {}. Initiating Migration.", self.epoch);
                    
                    // 1. 寻找新的物理常数 p'
                    let new_p = self.governor.migrate_algebra();
                    
                    // 2. [LIFTER] 灵魂转世：p -> p'
                    // 携带旧记忆 (Feature Invariants)，在新宇宙重塑肉身
                    self.state = self.lifter.lift_and_requantize(&self.state, new_p);
                    
                    // 3. 重置优化器动量
                    // 新空间的曲率特性完全不同，旧的动量或模式已失效
                    self.optimizer.mode = OptimizationMode::GradientFlow;
                    
                    println!("[System] Migration Complete. Resuming evolution in Cl(-{}).", new_p);
                    continue;
                }
            }

            // =============================================================
            // Phase 2: 动力学意图 (Dynamic Will)
            // =============================================================
            
            // 1. 感知时空曲率 (Sense Curvature)
            // 计算当前状态附近的离散 Ollivier-Ricci 曲率
            let kappa = self.state.calculate_ricci_curvature();
            
            // 2. 切换战术模式 (Switch Tactics)
            // GradientFlow (平坦/正曲率) vs HyperbolicBeam (负曲率/混乱)
            let _mode = self.optimizer.switch_mode(kappa);
            
            // 3. 生成邻域候选者 (Generate Candidates)
            let raw_candidates_states = self.state.generate_neighbors();
            
            // 建立 Hash -> State 的映射，因为 Optimizer 处理的是 u64 Hash
            let mut candidate_map = HashMap::new();
            let mut raw_candidates_hashes = Vec::new();
            
            for cand in raw_candidates_states {
                let h = cand.hash();
                candidate_map.insert(h, cand);
                raw_candidates_hashes.push(h);
            }
            
            // 4. 定义目标函数 (Objective Function)
            // 闭包捕获 evaluator 和 map
            let objective_fn = |hash: u64| -> f64 {
                if let Some(cand) = candidate_map.get(&hash) {
                    // 计算基础几何能量 (Sinkhorn Distance / L2)
                    self.evaluator.geometric_loss(cand)
                } else {
                    f64::MAX // Should not happen
                }
            };

            // 5. 执行优化步 (Step)
            // 优化器根据当前模式筛选出最有希望的下一代状态
            let best_hashes = self.optimizer.step(raw_candidates_hashes, objective_fn);
            
            // =============================================================
            // Phase 3: 卫士审查 (The Lazy Guard)
            // =============================================================
            
            if let Some(&best_hash) = best_hashes.first() {
                // 既然 Optimizer 选择了它，我们先取出状态
                let best_cand = candidate_map.get(&best_hash).unwrap();
                let geom_loss = self.evaluator.geometric_loss(best_cand);
                
                // 为了惰性审查，我们需要生成局部点云 (Embedding Cloud)
                // 这模拟了状态在特征空间中的微观分布
                let cloud: Vec<DVector<f64>> = best_cand.generate_embedding_cloud(20);

                // [GUARD] 惰性审查
                // 只有当 loss 很低 (诱惑) 且 kappa 很低 (风险) 时才触发同调检查
                let (veto, _penalty) = self.guard.inspect(&cloud, geom_loss, kappa);

                if !veto {
                    // [ACCEPT] 卫士放行
                    self.state = best_cand.clone();
                    
                    // [FINAL CHECK] 检查是否完全收敛 (Logical Zero)
                    // 这需要几何误差极小，且通过严格的 STP 代数验证 (verify_exact)
                    if geom_loss < 1e-6 && self.evaluator.verify_exact(&self.state) {
                        println!("[System] ✨ TRUTH DISCOVERED at Epoch {}. Energy ~ 0.", self.epoch);
                        return Ok(self.state.clone());
                    }
                } else {
                    // [REJECT] 卫士否决 (发现拓扑孔洞/逻辑死循环)
                    // println!("[System] Guard Vetoed candidate at Epoch {}. Injecting Entropy.", self.epoch);
                    
                    // 施加惩罚并强制随机扰动 (Entropy Injection / Random Jump)
                    // 这防止系统在拓扑陷阱周围震荡
                    self.state = self.state.random_jump();
                }
            } else {
                // [STUCK] 死胡同：没有生成有效的候选者
                // 可能是进入了孤立点或约束过强
                // println!("[System] Stuck at Epoch {}. Random Jump.", self.epoch);
                self.state = self.state.random_jump();
            }
        }
    }
}
