// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use crate::phase3::evolution::gene::{ProbeGene, ProbeState};
use crate::phase3::evolution::mutagen::{BiasVapo, PrimeAdaptive};
use crate::phase3::core::neuron::HTPNeuron;
use crate::phase3::core::affine::AffineTuple;
use crate::phase3::core::algebra::ClassGroupElement;
use crate::phase3::decoder::InverseDecoder;

use std::collections::{BinaryHeap, HashSet};
use std::sync::{Arc, RwLock};
use rug::Integer;

/// 🦖 EvolutionaryEngine: 达尔文引擎 (Dual-Track VAPO Enabled)
/// 
/// 实现了修正后的双轨进化架构：
/// - Track A (Bias): 基于超度量反馈的 VAPO 微调。
/// - Track B (Prime): 基于统计学习的自适应搜索。
pub struct EvolutionaryEngine {
    /// [Environment]: 代数环境
    neuron_core: Arc<RwLock<HTPNeuron>>,
    
    /// [Navigator]: 坐标解码器 (已升级为支持 Bias 和 CPL)
    decoder: InverseDecoder,
    
    /// [Mutators]: 双轨突变器
    bias_mutator: BiasVapo,       // Track A
    prime_mutator: PrimeAdaptive, // Track B

    /// [Time Machine]: 优先队列
    search_queue: BinaryHeap<ProbeState>,

    /// [History]: 已探索空间 (Tabu Search)
    visited_hashes: HashSet<u64>,

    /// [Parameters]
    precision_target: f64, 
    max_generations: usize,
    target_token_id: Option<u32>, // 当前搜索的目标 Token
}

impl EvolutionaryEngine {
    pub fn new(
        neuron: Arc<RwLock<HTPNeuron>>, 
        vocab_size: u32
    ) -> Self {
        // 假设 vocab tensor 维度为 4
        let dims = 4;
        
        EvolutionaryEngine {
            neuron_core: neuron,
            decoder: InverseDecoder::new(vocab_size),
            bias_mutator: BiasVapo::new(dims),
            prime_mutator: PrimeAdaptive::new(),
            search_queue: BinaryHeap::new(),
            visited_hashes: HashSet::new(),
            precision_target: 0.0, 
            max_generations: 1000,
            target_token_id: None,
        }
    }

    /// 设置当前的搜索目标
    pub fn set_target(&mut self, target_id: u32) {
        self.target_token_id = Some(target_id);
    }

    /// 🌪️ 主要进化循环：寻找真理
    pub fn evolve_until_optimality(&mut self, initial_state: AffineTuple) -> Result<ProbeGene, String> {
        self.seed_population(initial_state);

        let mut generation = 0;

        while let Some(parent_state) = self.search_queue.pop() {
            if generation > self.max_generations {
                return Err("Evolution Timeout.".to_string());
            }

            let parent_gene = &parent_state.gene;

            // 撞墙检测与目标检查
            match self.decoder.decode_with_bias(&parent_gene.current_state, &parent_gene.bias_vector) {
                Ok(result) => {
                    if (result.drift as f64) <= self.precision_target {
                        // 如果设置了特定目标 ID，还需检查 ID 是否匹配
                        if let Some(tid) = self.target_token_id {
                            if result.token_id == tid {
                                println!("🏆 Truth Found! Gen: {}, Depth: {}", generation, parent_gene.depth);
                                return Ok(parent_gene.clone());
                            }
                        } else {
                            // 无特定目标，只求无漂移
                            return Ok(parent_gene.clone());
                        }
                    }
                },
                Err(_) => {
                    continue; // 撞墙，放弃该分支
                }
            }

            // 裂变：生成子代
            let offspring = self.spawn_offspring(parent_gene);

            // 评估并入队
            for (child, mut_meta) in offspring {
                if let Some((scored_child, reward)) = self.evaluate_fitness(child) {
                    self.search_queue.push(scored_child);
                    
                    // 🔥 反馈回路 (Feedback Loop)
                    // 根据子代的表现，反向更新突变器的参数
                    match mut_meta {
                        MutationType::Bias { level } => {
                            self.bias_mutator.update_feedback(level, reward);
                        },
                        MutationType::Prime { strategy } => {
                            // 简单的二值奖励：如果 fitness 较高则算成功
                            // 这里阈值设为 0.5 仅作示例
                            let success = reward > 0.5;
                            self.prime_mutator.update_stats(strategy, success);
                        }
                    }
                }
            }

            generation += 1;
        }

        Err("Extinction.".to_string())
    }

    fn seed_population(&mut self, initial_state: AffineTuple) {
        let seed = ProbeGene {
            p_weight: Integer::from(1), 
            bias_vector: vec![0; 4],
            depth: 0,
            current_state: initial_state,
        };
        
        // 初始扩散
        // 此时还无法获得反馈，只进行生成
        let offspring = self.spawn_offspring(&seed);
        for (child, _) in offspring {
             if let Some((scored, _)) = self.evaluate_fitness(child) {
                 self.search_queue.push(scored);
             }
        }
    }

    /// 🧬 修正后的 spawn_offspring
    /// 实现了双轨生成逻辑
/// 🧬 修正后的 spawn_offspring
/// 双轨生成逻辑：
/// - Track A (Bias): 只改观测校准，不推进代数状态（避免重复幂演化）
/// - Track B (Prime): 推进代数状态（真正的“物理演化”）
    /// 🧬 修正后的 spawn_offspring
    /// 双轨生成逻辑：
    /// - Track A (Bias): 只改观测校准，不推进代数状态（避免重复幂演化）
    /// - Track B (Prime): 推进代数状态（真正的“物理演化”）
    fn spawn_offspring(&mut self, parent: &ProbeGene) -> Vec<(ProbeGene, MutationType)> {
        let mut offspring = Vec::new();
        let side_len = self.decoder.vocab_tensor.side_length;

        // 1) Track A: Bias VAPO (基于父代 P，微调 Bias)
        for _ in 0..3 {
            let mut new_bias = parent.bias_vector.clone();
            let level = self.bias_mutator.mutate(&mut new_bias, side_len);

            offspring.push((
                ProbeGene {
                    p_weight: parent.p_weight.clone(), // 不变：该状态对应的 P
                    bias_vector: new_bias,
                    depth: parent.depth + 1,
                    current_state: parent.current_state.clone(), // 不变：只改观测
                },
                MutationType::Bias { level },
            ));
        }

        // 2) Track B: Prime Adaptive (保持父代 Bias，探索 P，并推进状态)
        let strat = self.prime_mutator.select_strategy();
        let new_p = self.prime_mutator.generate(strat, &parent.p_weight);

        if let Ok(neuron_guard) = self.neuron_core.read() {
            let p_op = AffineTuple {
                p_factor: new_p.clone(),
                q_shift: ClassGroupElement::identity(&neuron_guard.discriminant),
            };

            if let Ok(new_state) = parent.current_state.compose(&p_op, &neuron_guard.discriminant) {
                offspring.push((
                    ProbeGene {
                        p_weight: new_p,
                        bias_vector: parent.bias_vector.clone(),
                        depth: parent.depth + 1,
                        current_state: new_state,
                    },
                    MutationType::Prime { strategy: strat },
                ));
            }
        }

        offspring
    }



    /// ⚖️ 修正后的 evaluate_fitness
    /// 核心改动：使用 decode_with_bias 并引入 CPL 奖励
    /// 返回: (ProbeState, NormalizedReward)
    /// ⚖️ 修正后的 evaluate_fitness
/// 核心改动：
/// - **不再**在此处推进代数状态（避免 Bias 轨重复幂演化）
/// - 仅做观测：decode_with_bias + (可选) CPL 奖励
/// 返回: (ProbeState, NormalizedReward)
    /// ⚖️ 修正后的 evaluate_fitness
    /// 核心改动：
    /// - **不再**在此处推进代数状态（避免 Bias 轨重复幂演化）
    /// - 仅做观测：decode_with_bias + (可选) CPL 奖励
    /// 返回: (ProbeState, NormalizedReward)
    fn evaluate_fitness(&self, gene: ProbeGene) -> Option<(ProbeState, f64)> {
        // 1. 观测 (Decoder with Bias)
        let res = self.decoder.decode_with_bias(&gene.current_state, &gene.bias_vector).ok()?;

        // 2. CPL 辅助指标 (仅当设置了目标 token 时才有意义)
        let mut cpl_score = 0.0;
        if let Some(target_id) = self.target_token_id {
            let target_coord = self.decoder.vocab_tensor.map_id_to_coord(target_id as u64);

            // 预测坐标 (Raw) -> (Biased)
            let predicted_raw = self.decoder.extract_coordinate(&gene.current_state);
            let mut biased_coord = predicted_raw.clone();
            let l = self.decoder.vocab_tensor.side_length;
            for (i, b) in gene.bias_vector.iter().enumerate().take(biased_coord.len()) {
                biased_coord[i] = (biased_coord[i] + (b % l)) % l;
            }

            let cpl = self.decoder.ultrametric_cpl_20bits(&biased_coord, &target_coord);
            cpl_score = (cpl as f64) / 20.0; // Normalize to [0, 1]
        }

        // 3. 综合 Fitness
        let drift_score = 1.0 / (1.0 + res.drift as f64);

        // 避免无限追深：对深度加入轻微惩罚
        let depth_penalty = (gene.depth as f64) * 0.001;

        // Drift 越小越好，CPL 越大越好
        let fitness = drift_score * 0.7 + cpl_score * 0.3 - depth_penalty;

        Some((
            ProbeState { gene, fitness_score: fitness },
            fitness,
        ))
    }


}

/// 辅助枚举，用于记录突变类型以便反馈
enum MutationType {
    Bias { level: usize },
    Prime { strategy: u8 },
}
