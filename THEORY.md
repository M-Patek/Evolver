# HYPER-TENSOR PROTOCOL (HTP)：理论推导

## 摘要
HTP 使用两类聚合算子：
- 时间方向：采用**非交换**的“仿射演化（Affine Evolution）”来保证顺序敏感与历史不可重写；
- 空间方向：采用**交换/可结合**的 Abelian 聚合来保证“正交验证/折叠顺序不影响全局根（Orthogonal Consistency）”。

---

## 0. 记号与基本环境
- 工作于某个乘法群（例如类群）\(\mathcal{G}\) 上，群运算记为乘法“\(\cdot\)”，单位元为 \(1\)。
- \(\Delta\) 表示模数/判别式等系统参数；文中写作“\(\bmod \Delta\)”仅表示群元素的等价类表示。
- 时间步 \(t=1,2,\dots\)：
  - \(P_t\)：与该步绑定的参数（常见为素数/挑战值等，取值域使指数运算合法）
  - \(h_t\)：该步绑定的哈希/摘要
  - \(G\in \mathcal{G}\)：公共基元（或公共生成元样式的固定元素）

---

## 1. 时间算子：非交换演化（Affine Evolution）

### 1.1 状态转移定义
令 \(S_t \in \mathcal{G}\) 表示第 \(t\) 步的时间状态，定义单步转移为
\[
S_t \;=\; S_{t-1}^{P_t}\cdot G^{h_t}\pmod{\Delta}.
\]
其中 \(S\mapsto S^{P_t}\) 是对群元素的指数作用；乘上 \(G^{h_t}\) 作为“顺序敏感扰动/注入项”。

为便于聚合，将注入项记作
\[
Q_t \;:=\; G^{h_t}\in\mathcal{G}.
\]

---

### 1.2 仿射元组与其作用（Affine Tuple Action）
定义一个“时间仿射元组”
\[
\mathcal{A}=(P,Q),\quad P\in \mathbb{Z},\; Q\in \mathcal{G}.
\]
它对状态 \(S\in\mathcal{G}\) 的作用定义为
\[
\rho(\mathcal{A},S)\;:=\;S^P\cdot Q.
\]
于是单步转移可写为
\[
S_t=\rho\big((P_t,Q_t),\,S_{t-1}\big).
\]

---

### 1.3 仿射合成律（Composition Law）推导：\(\oplus_{\text{time}}\)
目标：将连续两步 \(\mathcal{A}_1=(P_1,Q_1)\)、\(\mathcal{A}_2=(P_2,Q_2)\) 合并为一个元组 \(\mathcal{A}_{12}\)，使得对任意输入状态 \(S\)：
\[
\rho(\mathcal{A}_{12},S)=\rho\big(\mathcal{A}_2,\rho(\mathcal{A}_1,S)\big).
\]

展开右侧：
\[
\begin{aligned}
\rho\big(\mathcal{A}_2,\rho(\mathcal{A}_1,S)\big)
&=\rho\big((P_2,Q_2),\, S^{P_1}\cdot Q_1\big) \\
&=(S^{P_1}\cdot Q_1)^{P_2}\cdot Q_2 \\
&=S^{P_1P_2}\cdot Q_1^{P_2}\cdot Q_2.
\end{aligned}
\]
（这里使用了指数对乘法的分配：\((XY)^{P_2}=X^{P_2}Y^{P_2}\)，以及群运算结合律。）

因此只要令
\[
\mathcal{A}_{12}=(P_1P_2,\; Q_1^{P_2}\cdot Q_2),
\]
就有
\[
\rho(\mathcal{A}_{12},S)=S^{P_1P_2}\cdot Q_1^{P_2}\cdot Q_2
\]
与上式一致。

于是定义时间合并算子（注意：通常**非交换**）
\[
(P_1,Q_1)\;\oplus_{\text{time}}\;(P_2,Q_2)
\;:=\;
(P_1P_2,\; Q_1^{P_2}\cdot Q_2).
\]

---

### 1.4 结合律证明：\(\oplus_{\text{time}}\) 构成幺半群（Monoid）
令 \(\mathcal{A}_i=(P_i,Q_i)\)。分别计算两种加括号方式。

**左结合：**
\[
\begin{aligned}
(\mathcal{A}_1\oplus_{\text{time}}\mathcal{A}_2)\oplus_{\text{time}}\mathcal{A}_3
&=
(P_1P_2,\;Q_1^{P_2}Q_2)\oplus_{\text{time}}(P_3,Q_3) \\
&=
\big(P_1P_2P_3,\;(Q_1^{P_2}Q_2)^{P_3}\cdot Q_3\big) \\
&=
\big(P_1P_2P_3,\;Q_1^{P_2P_3}\cdot Q_2^{P_3}\cdot Q_3\big).
\end{aligned}
\]

**右结合：**
\[
\begin{aligned}
\mathcal{A}_1\oplus_{\text{time}}(\mathcal{A}_2\oplus_{\text{time}}\mathcal{A}_3)
&=
(P_1,Q_1)\oplus_{\text{time}}(P_2P_3,\;Q_2^{P_3}Q_3) \\
&=
\big(P_1P_2P_3,\;Q_1^{P_2P_3}\cdot (Q_2^{P_3}Q_3)\big) \\
&=
\big(P_1P_2P_3,\;Q_1^{P_2P_3}\cdot Q_2^{P_3}\cdot Q_3\big).
\end{aligned}
\]

两者完全一致，因此
\[
(\mathcal{A}_1\oplus_{\text{time}}\mathcal{A}_2)\oplus_{\text{time}}\mathcal{A}_3
=
\mathcal{A}_1\oplus_{\text{time}}(\mathcal{A}_2\oplus_{\text{time}}\mathcal{A}_3).
\]
故 \((\{\,(P,Q)\,\},\oplus_{\text{time}})\) 是一个**幺半群**。

**单位元：** 取
\[
\mathcal{I}=(1,1),
\]
则对任意 \((P,Q)\)：
\[
\mathcal{I}\oplus_{\text{time}}(P,Q)=(P,Q),\quad (P,Q)\oplus_{\text{time}}\mathcal{I}=(P,Q).
\]
（后者需要 \(Q^{1}=Q\) 且乘上 \(1\) 不改变元素。）

---

### 1.5 区间聚合（用于并行/Segment Tree）
对一段连续时间区间 \([l,r]\)，定义其聚合元组为
\[
\mathcal{A}_{[l,r]} \;:=\; (P_l,Q_l)\oplus_{\text{time}}(P_{l+1},Q_{l+1})\oplus_{\text{time}}\cdots\oplus_{\text{time}}(P_r,Q_r).
\]
由结合律可知可以任意括号化，从而支持二叉树/线段树式的并行构造与 \(O(\log n)\) 查询合并。

区间对初始状态 \(S_{l-1}\) 的作用为
\[
S_r = \rho(\mathcal{A}_{[l,r]},\, S_{l-1}).
\]

---

## 2. 空间算子：交换聚合（Abelian Aggregation）

### 2.1 设计目标
空间折叠用于“全局锚定/全局根”的一致性验证，关键要求是：
- 对同一批空间单元的聚合结果，不依赖折叠顺序与括号结构（可交换 + 可结合）。

因此空间侧**不再**使用非交换的 \(\oplus_{\text{time}}\)；而是对空间贡献采用 Abelian 聚合。

### 2.2 空间贡献与聚合算子 \(\otimes_{\text{space}}\)
对每个空间单元（例如网格坐标 \(\vec{v}\)）先在其时间轴上聚合得到时间元组
\[
\mathcal{A}_{\vec{v}}=(P_{\vec{v}},Q_{\vec{v}}),
\]
其中 \(Q_{\vec{v}}\in\mathcal{G}\) 作为该单元参与空间锚定的“空间贡献”。

定义空间聚合为群乘法：
\[
X\otimes_{\text{space}}Y \;:=\; X\cdot Y,\quad X,Y\in\mathcal{G}.
\]
若 \(\mathcal{G}\) 为 Abelian 群，则：
- 交换：\(X\cdot Y=Y\cdot X\)
- 结合：\((X\cdot Y)\cdot Z=X\cdot (Y\cdot Z)\)

于是任意空间集合 \(\mathcal{V}\) 的全局空间根可写为
\[
R_{\text{space}} \;:=\; \bigotimes_{\vec{v}\in\mathcal{V}} Q_{\vec{v}}
\;=\;
\prod_{\vec{v}\in\mathcal{V}} Q_{\vec{v}}.
\]

---

## 3. 正交一致性（Orthogonal Consistency / Fold-Order Invariance）
设空间折叠的目标是把同一批 \(\{Q_{\vec{v}}\}\) 聚合为同一个根 \(R_{\text{space}}\)。

由于 \(\otimes_{\text{space}}\) 是交换且可结合的，
- 任意折叠路径（先按行后按列、先按列后按行、或任意多叉树括号化）都只是在对同一多重集合做乘法；
- 因此结果恒等于 \(\prod_{\vec{v}} Q_{\vec{v}}\)，与折叠顺序无关。

形式化地，对任意两个遍历/折叠顺序 \(\pi,\pi'\)：
\[
\prod_{i} Q_{\pi(i)} \;=\; \prod_{i} Q_{\pi'(i)}.
\]
这就是“正交验证不受折叠轴/括号结构影响”的数学基础。

---

## 4. 安全性简述（高层）
- 时间侧安全性：依赖隐藏阶/强根等假设下的困难性，使得攻击者难以在保持锚定一致的同时重排/篡改历史序列（顺序敏感由非交换仿射演化提供）。
- 空间侧安全性：空间根是对空间贡献的 Abelian 聚合；伪造相当于在底层群假设下构造不一致但可通过验证的贡献集合，通常可归约到根问题/强 RSA 类假设（视具体群与参数化而定）。

---
