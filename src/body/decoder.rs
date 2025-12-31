use crate::soul::algebra::ClassGroupElement;
use crate::body::topology::VPuNNConfig;
use crate::body::projection::project_to_digit;

/// 路径实体化 (Path Materialization)
///
/// 将一个潜意识的代数种子 (ClassGroupElement) 展开为一条具体的显意识决策路径 (v-PuNN Path)。
/// 
/// [Architecture Update]: 
/// 采用了“分形展开”策略，传递了 `layer` 参数给投影函数。
/// 这实现了 \Psi_k(S) = (a + k*b) mod P 的分形投影。
pub fn materialize_path(state: &ClassGroupElement, config: &VPuNNConfig) -> Vec<u64> {
    // 1. 克隆初始状态
    let mut current_state = state.clone();
    
    let mut path = Vec::with_capacity(config.depth);

    // 2. 逐层展开 (Layer-wise Unfolding)
    for layer in 0..config.depth {
        // A. 投影 (Projection): 获取当前层级的决策数字
        // 关键修正：传入 layer 作为 depth 参数
        let digit = project_to_digit(&current_state, config.p_base, layer as u64);
        path.push(digit);

        // B. 演化 (Evolution): 状态自旋 (Self-Composition / Squaring)
        // S_{k+1} = S_k ∘ S_k
        // 这种幂运算在代数图上提供了“混合”(Mixing) 效果，
        // 配合线性的投影函数，构成了类似混沌系统的动力学特征。
        let next_state = current_state.compose(&current_state);
        
        current_state = next_state;
    }

    path
}
