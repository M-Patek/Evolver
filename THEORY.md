# HYPER-TENSOR PROTOCOL (HTP): Core Protocol Specification

## Abstract

This document defines the Hyper-Tensor Protocol (HTP) v1.0. It establishes the Dual-Operator Algebra used to verify the causal integrity (Time) and holographic consistency (Space) of the Evolver system.

**Note on Architecture:** While the underlying Generator is a probabilistic neural network, this protocol treats the system as a deterministic algebraic state machine. The bridge between the neural logic and this protocol is defined in the Bias-Control Interface (Section 5).

---

## 1. The Time Operator: Non-Commutative Evolution

### 1.1 Problem Definition

HTP enforces order sensitivity in the temporal dimension. History cannot be rewritten.

Let $S_t$ be the state at step $t$. The state transition is defined as:

$$S_t = \mathcal{F}(S_{t-1}, P_t, h_t) = S_{t-1}^{P_t} \cdot G^{h_t} \pmod \Delta$$

Where:
* $P_t$: Prime representative of the event/token at step $t$.
* $h_t$: Hash of the spacetime depth $H(t)$.
* $G$: Generator of the class group.

### 1.2 Derivation of Time Composition Law ($\oplus_{\text{time}}$)

We define the affine tuple $\mathcal{A} = (P, Q)$ acting on state $S$ as $\rho(\mathcal{A}, S) = S^P \cdot Q$. For two consecutive transformations $\mathcal{A}_1 = (P_1, Q_1)$ and $\mathcal{A}_2 = (P_2, Q_2)$, the merged operator is derived as:

$$\begin{aligned} \rho(\mathcal{A}_2, \rho(\mathcal{A}_1, S)) &= (S^{P_1} \cdot Q_1)^{P_2} \cdot Q_2 \\ &= S^{P_1 P_2} \cdot (Q_1^{P_2} \cdot Q_2) \end{aligned}$$

Thus, the Time Operator is defined as:

$$\mathcal{A}_1 \oplus_{\text{time}} \mathcal{A}_2 = (P_1 \cdot P_2, \quad Q_1^{P_2} \cdot Q_2)$$

### 1.3 Proof of Associativity

For Segment Trees to function, the operator must be associative: $(\mathcal{A}_1 \oplus \mathcal{A}_2) \oplus \mathcal{A}_3 \equiv \mathcal{A}_1 \oplus (\mathcal{A}_2 \oplus \mathcal{A}_3)$.

**Left Side:** $(\mathcal{A}_1 \oplus \mathcal{A}_2) \oplus \mathcal{A}_3$

$$= (P_1 P_2, Q_1^{P_2} Q_2) \oplus (P_3, Q_3)$$
$$= (P_1 P_2 P_3, \quad (Q_1^{P_2} Q_2)^{P_3} Q_3)$$
$$= (P_1 P_2 P_3, \quad Q_1^{P_2 P_3} Q_2^{P_3} Q_3)$$

**Right Side:** $\mathcal{A}_1 \oplus (\mathcal{A}_2 \oplus \mathcal{A}_3)$

$$= (P_1, Q_1) \oplus (P_2 P_3, Q_2^{P_3} Q_3)$$
$$= (P_1 (P_2 P_3), \quad Q_1^{P_2 P_3} (Q_2^{P_3} Q_3))$$
$$= (P_1 P_2 P_3, \quad Q_1^{P_2 P_3} Q_2^{P_3} Q_3)$$

**Conclusion:** The Time Operator is Associative but Non-Commutative.

---

## 2. The Space Operator: Commutative Aggregation

### 2.1 The Dimensional Requirement

To ensure holographic consistency (the ability to verify from any axis), spatial aggregation must be Commutative.

### 2.2 Derivation of Space Composition Law ($\otimes_{\text{space}}$)

We leverage the intrinsic Abelian property of the Class Group $Cl(\Delta)$ and integer multiplication. We define the Space Operator as component-wise aggregation:

$$\mathcal{A}_1 \otimes_{\text{space}} \mathcal{A}_2 = (P_1 \cdot P_2, \quad Q_1 \cdot Q_2)$$

Where $Q_1 \cdot Q_2$ is standard group multiplication.

### 2.3 Proof of Commutativity

Since $\mathbb{Z}$ and $Cl(\Delta)$ are Abelian:
* $P_1 \cdot P_2 = P_2 \cdot P_1$
* $Q_1 \cdot Q_2 = Q_2 \cdot Q_1$

Therefore:

$$\mathcal{A}_1 \otimes_{\text{space}} \mathcal{A}_2 = \mathcal{A}_2 \otimes_{\text{space}} \mathcal{A}_1$$

---

## 3. Hyper-Tensor Folding

The Hyper-Tensor $\mathcal{T}$ uses a hybrid topology:
* **Micro-Cells (Time):** Internal neuron history is aggregated via $\oplus_{\text{time}}$.
* **Macro-Grid (Space):** Tensor dimensions are folded via $\otimes_{\text{space}}$.

### 3.1 The Folding Operator $\Phi$

For a tensor of dimension $d$, folding along dimension $k$ uses the Space Operator:

$$\text{Fold}_k(\mathcal{T}) = \bigotimes_{i=1}^{L} \mathcal{T}_{(i, \dots)}$$

### 3.2 Proof of Orthogonal Consistency

We assert that for any two axes $x, y$, the order of folding does not matter:

$$\text{Fold}_y(\text{Fold}_x(\mathcal{T})) \equiv \text{Fold}_x(\text{Fold}_y(\mathcal{T}))$$

**Proof:**
Let $\mathcal{T}_{ij}$ be the element at coordinate $x=i, y=j$.

**LHS:**

$$\text{Fold}_y \left( \bigotimes_i \mathcal{T}_{ij} \right) = \bigotimes_j \left( \bigotimes_i \mathcal{T}_{ij} \right) = \prod_j \prod_i \mathcal{T}_{ij}$$

**RHS:**

$$\text{Fold}_x \left( \bigotimes_j \mathcal{T}_{ij} \right) = \bigotimes_i \left( \bigotimes_j \mathcal{T}_{ij} \right) = \prod_i \prod_j \mathcal{T}_{ij}$$

Since the product is over a finite Abelian group (Space Operator), the terms can be reordered arbitrarily. Thus, LHS $\equiv$ RHS.

**Q.E.D.**

---

## 4. Security Assumptions

### 4.1 Time Security (Hidden Order)

Security relies on the infeasibility of finding the order of $Cl(\Delta)$. An adversary cannot forge a history proof $(W, R)$ such that $W^P \cdot R \equiv T$ without solving the discrete log problem.

### 4.2 Space Security (Adaptive Root)

Forging a spatial inclusion proof requires solving the root problem $X^e \equiv Y \pmod \Delta$ (Strong RSA Assumption equivalent).

### 4.3 The Kernel Trap (Boundary Analysis)

**Mathematical Possibility:**
While $\oplus_{\text{time}}$ generally ensures perturbation propagation, a "Kernel Trap" exists if the perturbation $\varepsilon \neq 1$ falls into the kernel of the power map $x \mapsto x^P$:

$$\varepsilon^P \equiv 1 \pmod \Delta$$

This requires $\text{ord}(\varepsilon) \mid P$.

**Engineering Mitigation:**
* **Huge Class Number:** The discriminant size ($\ge 2048$ bits) implies $|\Delta| \approx 2^{2048}$, making accidental collision with small-order elements negligible.
* **Large Primes:** $P$ is a large prime (e.g., 64-bit). For $\text{ord}(\varepsilon) \mid P$, the order must be exactly $P$. Finding such an element without knowing the class number is computationally infeasible.

---

## 5. The Bias-Control Interface (Effective Model)

For the historical derivation of exact controllability, see *Security vs. Trainability.md*.

### 5.1 The Control Problem

The HTP Protocol requires the system to output a specific Algebraic Root $Q_{target}$ that satisfies logical constraints ($Energy = 0$). However, the Generator outputs a probabilistic vector (Logits).

We define the Effective Control Model as a function that maps a control signal $\vec{b}$ to a Token selection.

### 5.2 The Unified Bias Definition

The Bias Vector $\vec{b}$ operates across three layers of abstraction:

* **Layer 0 (Control State):** $\vec{b}_{ctrl} \in (\mathbb{Z}/L\mathbb{Z})^{16}$. The discrete object optimized by VAPO.
* **Layer 1 (Embedding):** $\phi(\vec{b}_{ctrl}) \in \mathbb{R}^{32}$. A continuous cyclic embedding preserving topology.
* **Layer 2 (Projection):** $\vec{b}_{logits} = W_{proj} \cdot \phi(\vec{b}_{ctrl})$. The force applied to the neural manifold.

### 5.3 Protocol Binding

A valid HTP Proof Bundle must commit to both the Algebra and the Control Signal:

$$\text{ProofBundle} := \{ \text{GlobalRoot}_{\text{alg}}, \vec{b}_{ctrl}, \text{Proof}_{\text{validity}} \}$$

The verification logic holds if and only if:

$$\text{Action} = \text{Argmax}( \text{Logits}_{gen} + \text{Project}(\vec{b}_{ctrl}) )$$

This ensures that the "correction" applied to the logic is explicitly revealed and cryptographically bound to the context.
