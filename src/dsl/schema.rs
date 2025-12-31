use crate::dsl::schema::{ProofAction, LogicType};

/// 语义适配器 (Semantic Adapter)
///
/// 职责：将代数投影产生的“无意义”数字序列，确定性地转换为“有意义”的逻辑证明动作序列。
/// 这是一个核心的架构组件，它连接了 "Body" (拓扑投影) 和 "Mind" (逻辑验证)。
///
/// 映射逻辑 (v1.5 Parity Logic):
/// - Digits[0] -> 变量 'n' 的初始类型 (Even/Odd)
/// - Digits[1] -> 变量 'm' 的初始类型 (Even/Odd)
/// - Digits[2] -> 目标断言的类型 (Even/Odd)
pub struct SemanticAdapter;

impl SemanticAdapter {
    /// 将一串 u64 (来自代数状态投影) 转化为具体的证明步骤
    ///
    /// # 参数
    /// * `path`: 来自 BodyProjector 的数字序列
    ///
    /// # 返回
    /// * `Vec<ProofAction>`: 可被 STP 引擎执行的逻辑动作序列
    pub fn materialize(path: &[u64]) -> Vec<ProofAction> {
        let mut actions = Vec::new();

        // 鲁棒性检查：如果路径太短，无法形成最小的逻辑三段论，返回空序列。
        // 空序列在 STP Bridge 中会被赋予极高的能量惩罚。
        if path.len() < 3 {
            return actions;
        }

        // --- Step 1: 定义前置条件 (Premises) ---
        
        // 利用 path[0] 的奇偶性定义第一个变量 'n'
        // 奇数映射为 Odd，偶数映射为 Even
        let type_n = if path[0] % 2 == 0 { LogicType::Even } else { LogicType::Odd };
        actions.push(ProofAction::Define { 
            symbol: "n".to_string(), 
            initial_type: type_n 
        });

        // 利用 path[1] 的奇偶性定义第二个变量 'm'
        // 这迫使 VAPO 搜索必须同时满足两个变量的代数约束
        let type_m = if path[1] % 2 == 0 { LogicType::Even } else { LogicType::Odd };
        actions.push(ProofAction::Define { 
            symbol: "m".to_string(), 
            initial_type: type_m 
        });

        // --- Step 2: 构造结论 (Conclusion) ---

        // 利用 path[2] 决定断言的目标类型
        // 这是一个 "猜测" (Hypothesis)。
        // 只有当这个猜测与 Step 1 的物理推导结果一致时，能量才会归零。
        let target_type = if path[2] % 2 == 0 { LogicType::Even } else { LogicType::Odd };
        
        // 构造断言语句： Assert "(n + m) is [TargetType]"
        // 在 STP 内部，这会触发隐式的 Apply(Add, n, m) 并检查结果
        actions.push(ProofAction::Assert { 
            condition: format!("(n + m) is {:?}", target_type) 
        });

        // (未来扩展)
        // 如果 path 更长，可以继续通过 adapter 映射更多步骤，例如：
        // path[3] -> Apply(Multiply, n, m)
        // path[4] -> Assert result is ...
        
        actions
    }
}
