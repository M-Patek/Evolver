use std::collections::HashMap;
use crate::dsl::schema::ProofAction;
use crate::dsl::math_kernel::Matrix; // 引入严格的 STP 内核

pub struct StpContext {
    // 状态矩阵：现在是一个严格的 MathMatrix，而不是 Vec<Vec<f64>>
    pub state_matrix: Matrix,
    pub energy: f64,
    // 变量映射：符号名 -> 逻辑矩阵
    pub variables: HashMap<String, Matrix>,
}

impl StpContext {
    pub fn new() -> Self {
        StpContext {
            // 初始化为一个 1x1 的单位状态 [1.0]
            state_matrix: Matrix::identity(1),
            energy: 0.0,
            variables: HashMap::new(),
        }
    }

    /// 执行一步 STP 演化
    /// State(t+1) = Operator |x| State(t)
    pub fn evolve(&mut self, operator: &Matrix) {
        // 这里是关键喵！
        // 旧代码可能直接用了 operator * state (MatMul)
        // 或者 operator (x) state (Kron)
        // 现在的逻辑：严格使用 Semi-Tensor Product
        
        // 左乘算子：L |x| x
        self.state_matrix = operator.stp(&self.state_matrix);
        
        // 每次演化后，我们可以计算能量（这里简化为状态向量的熵或模长，视具体物理定义而定）
        self.calculate_energy();
    }

    /// 定义变量
    pub fn define_variable(&mut self, name: &str, value: Matrix) {
        self.variables.insert(name.to_string(), value);
    }

    /// 简单的能量计算模拟
    fn calculate_energy(&self) {
        // 在真正的 STP 逻辑中，如果状态归约到了逻辑矛盾（例如零向量），能量应激增
        // 这里仅作演示：检查是否为零矩阵
        let sum: f64 = self.state_matrix.data.iter().map(|x| x.abs()).sum();
        if sum < 1e-6 {
            self.energy = 100.0; // 逻辑矛盾（崩溃）
        } else {
            self.energy = 0.0;   // 逻辑自洽
        }
    }
    
    // 获取当前状态用于调试
    pub fn get_state_dims(&self) -> (usize, usize) {
        (self.state_matrix.rows, self.state_matrix.cols)
    }
}

// 辅助函数：把逻辑值转换为 Matrix (Logical Vector)
// 例如：True -> [1, 0], False -> [0, 1] (如果是 2-valued logic)
pub fn bool_to_matrix(val: bool) -> Matrix {
    if val {
        Matrix::new(2, 1, vec![1.0, 0.0]) // True
    } else {
        Matrix::new(2, 1, vec![0.0, 1.0]) // False
    }
}
