use crate::dsl::schema::{ProofAction, LogicType, LogicValue};

/// 语义适配器：将代数投影产生的数字序列转化为逻辑动作序列
/// 这里实现了文档中描述的 "Adapter Pattern"
pub struct SemanticAdapter;

impl SemanticAdapter {
    /// 将一串 u64 (来自代数状态投影) 转化为具体的证明步骤
    /// 
    /// 映射逻辑 (根据 path 的不同位决定不同的动作):
    /// - step 0: 定义变量 n 的类型 (Odd/Even)
    /// - step 1: 定义变量 m 的类型 (Odd/Even)
    /// - step 2: 决定操作类型 (Assert Sum is Even / Odd)
    /// 
    /// 这样，VAPO 必须同时优化这三个维度的参数才能使能量归零。
    pub fn materialize(path: &[u64]) -> Vec<ProofAction> {
        let mut actions = Vec::new();

        // 如果路径太短，返回空，这将导致 STP 能量极高
        if path.len() < 3 {
            return actions;
        }

        // --- Step 1: 利用 path[0] 定义第一个变量 'n' ---
        let type_n = if path[0] % 2 == 0 { LogicType::Even } else { LogicType::Odd };
        actions.push(ProofAction::Define { 
            symbol: "n".to_string(), 
            initial_type: type_n 
        });

        // --- Step 2: 利用 path[1] 定义第二个变量 'm' ---
        // 注意：这里引入了 path[1]，VAPO 必须搜索第二维度的扰动
        let type_m = if path[1] % 2 == 0 { LogicType::Even } else { LogicType::Odd };
        actions.push(ProofAction::Define { 
            symbol: "m".to_string(), 
            initial_type: type_m 
        });

        // --- Step 3: 利用 path[2] 决定断言的目标 ---
        // 只有当 path[2] 映射为 "Even" 时，且 n, m 确实满足加法规则，能量才会为 0
        let target_type = if path[2] % 2 == 0 { LogicType::Even } else { LogicType::Odd };
        
        // 构造一个断言： Assert (n + m) is target_type
        // 在 STP 内部，这会触发 Apply(Add, n, m) 然后检查结果是否匹配 target_type
        actions.push(ProofAction::Assert { 
            condition: format!("(n + m) is {:?}", target_type) 
        });

        actions
    }
}
