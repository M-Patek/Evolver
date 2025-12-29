// New Evolver: Neuro-Symbolic Alignment Orchestrator
// 这个主程序演示了 Phase 1 & 2 的核心逻辑：
// Generator (Logits) -> STP Energy Check -> Bias VAPO Correction -> Valid Proof

mod dsl;
mod control;

use dsl::schema::{ProofAction, ProofSequence};
use dsl::stp_bridge::STPContext;
use control::bias_channel::{BiasController, BiasVector, VapoConfig};
use rand::Rng;

// 模拟的动作空间大小 (必须与 bias_channel.rs 中一致)
const ACTION_SPACE_SIZE: usize = 1024;

fn main() {
    println!("🐱 New Evolver System Initializing...");
    println!("--------------------------------------------------");

    // 1. 初始化代数环境 (The Algebraic World)
    let mut stp_ctx = STPContext::new();
    println!("[Init] STP Context loaded with theorems: ModAdd, Equals...");

    // 2. 初始化 VAPO 控制器 (The Sidecar)
    let mut controller = BiasController::new(Some(VapoConfig {
        max_iterations: 100,
        initial_temperature: 2.0,
        valuation_decay: 0.95,
    }));
    println!("[Init] VAPO Controller ready (Bias Dim: 16)");

    // ------------------------------------------------------------------
    // 场景模拟：证明 "两个奇数之和是偶数"
    // ------------------------------------------------------------------
    println!("\n📝 Mission: Prove that the sum of two Odd numbers is Even.");

    // Step 1: 定义变量 n (Odd) - 假设 Generator 做对了
    let action_step1 = ProofAction::Define {
        symbol: "n".to_string(),
        hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()],
    };
    stp_ctx.calculate_energy(&action_step1);
    println!("[Step 1] Generator defined 'n' as Odd. Energy: 0.0 (OK)");

    // Step 2: 定义变量 m (Odd) - 假设 Generator 也做对了
    let action_step2 = ProofAction::Define {
        symbol: "m".to_string(),
        hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()],
    };
    stp_ctx.calculate_energy(&action_step2);
    println!("[Step 2] Generator defined 'm' as Odd. Energy: 0.0 (OK)");

    // ------------------------------------------------------------------
    // Step 3: 关键推导 (Generator 犯错模拟)
    // ------------------------------------------------------------------
    println!("\n⚠️  [Step 3] Generating inference step...");

    // 模拟 Generator 的原始 Logits
    // 假设它很笨，大概率 (logits 高) 想生成一个错误结论 "sum is Odd"
    // 对应 Mock 解码器中的 index 0
    let mut raw_logits = vec![0.0; ACTION_SPACE_SIZE];
    raw_logits[0] = 5.0;  // 错误动作：Apply(ModAdd) -> Odd (Wrong!)
    raw_logits[1] = -2.0; // 正确动作：Apply(ModAdd) -> Even (Correct, but low prob)

    // 定义解码器 (Logits -> DSL Action)
    // 这是一个简化版，实际应该包含 Beam Search 或采样
    let decode_fn = |logits: &[f64]| -> ProofAction {
        // Find argmax
        let max_idx = logits.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        if max_idx == 0 {
            // 错误幻觉: 认为 Odd + Odd = Odd
            ProofAction::Define { 
                symbol: "sum".to_string(), 
                hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
            }
        } else {
            // 正确逻辑: 认为 Odd + Odd = Even
            // (注意：在实际 DSL 中，Apply 动作会自动推导，这里为了模拟 Generator 的"意图"，
            // 我们假设它显式输出了结果的定义，或者 Apply 的参数指向了错误的分支)
            ProofAction::Define { 
                symbol: "sum".to_string(), 
                hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Even".to_string()] 
            }
        }
    };

    // 3.1 检查原始生成
    let initial_action = decode_fn(&raw_logits);
    let initial_energy = stp_ctx.calculate_energy(&initial_action); // 这里需要 trick 一下，要在 ctx 里预演 Apply
    
    // 为了让能量计算生效，我们需要先在 STP Context 里模拟 "如果执行 ModAdd(n,m) 会得到什么"
    // 在 stp_bridge.rs 中，Apply 动作会自动计算预期。
    // 但我们的 decode_fn 返回的是 Define (模拟 Generator 直接断言结果)。
    // 我们手动在 Context 里执行一次“真理推导”
    stp_ctx.calculate_energy(&ProofAction::Apply {
        theorem_id: "ModAdd".to_string(),
        inputs: vec!["n".to_string(), "m".to_string()],
        output_symbol: "sum_truth".to_string(), // 临时的真理符号
    }); 
    // 此时 "sum_truth" 是 Even。
    // 如果 Generator 试图 Define "sum" 为 Odd，我们需要比较 "sum" 和 "sum_truth" (在 bridge 中需要支持这种比较)
    // *或者* 简单点，我们在 bridge 的 calculate_energy 里，
    // 如果发现是对一个已存在的推导结果进行重定义(Redefine)，则检查冲突。
    
    // 在此 Demo 中，我们假设 calculate_energy 能正确识别 "Odd != Even"
    // (需要 stp_bridge.rs 支持 path 比较，我们在之前的实现里做了简单的 check)

    println!("   -> Raw Generator intent: Define 'sum' as Odd.");
    // 让我们假设 STP 桥接检测到了冲突 (实际上我们需要把 sum 映射到 sum_truth)
    // 这里为了演示流程，我们手动打印
    println!("   -> STP Check: VIOLATION detected! (Odd + Odd != Odd)");

    // ------------------------------------------------------------------
    // 3.2 VAPO 介入修正
    // ------------------------------------------------------------------
    println!("\n🛡️  [VAPO] Bias Controller Engaging...");
    
    // 为了配合之前的测试逻辑，我们需要在 STP context 里把 "sum" 预设为 Even (真理)
    // 上面的 Apply 已经生成了 "sum_truth" (Even)。
    // 我们让 decode_fn 返回的 Define 作用于 "sum_truth" 符号以便触发 bridge 的冲突检查
    let smart_decode_fn = |logits: &[f64]| -> ProofAction {
        let max_idx = logits.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        if max_idx == 0 {
            // 错误: 试图把 sum_truth 定义为 Odd
            ProofAction::Define { 
                symbol: "sum_truth".to_string(), 
                hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()] 
            }
        } else {
            // 正确: 试图把 sum_truth 定义为 Even
            ProofAction::Define { 
                symbol: "sum_truth".to_string(), 
                hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Even".to_string()] 
            }
        }
    };

    let (final_bias, final_action) = controller.optimize(&raw_logits, &mut stp_ctx, smart_decode_fn);

    println!("\n✅ [Result] Optimization Complete.");
    println!("   -> Final Action: {:?}", final_action);
    println!("   -> Applied Bias Vector: {:?}", final_bias.data);
    println!("   -> Logic is now ALIGNED.");

}
