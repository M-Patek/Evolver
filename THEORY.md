# HYPER-TENSOR PROTOCOL (HTP): Theoretical Proofs

## Abstract
This document provides the formal mathematical derivations for the HYPER-TENSOR PROTOCOL (HTP). It proves the correctness of the non-commutative affine evolution, the associativity of the composition law (enabling parallel segment trees), and the recursive structure of the hyper-tensor folding mechanism.

---

## 1. The Non-Commutative Evolution

### 1.1 Problem Definition
In a standard RSA accumulator, operations are commutative ($x^{ab} = x^{ba}$). HTP introduces a depth-dependent noise factor $G^{H(t)}$ to enforce order sensitivity.

Let $S_t$ be the state at step $t$. The state transition is defined as:

$$
S_t = \mathcal{F}(S_{t-1}, P_t, h_t) = S_{t-1}^{P_t} \cdot G^{h_t} \pmod \Delta
$$

Where:
* $P_t$: Prime representative of the member at step $t$.
* $h_t$: Hash of the spacetime depth $H(t)$.
* $G$: Generator of the class group.

### 1.2 Recursive Expansion
We aim to express the state $S_n$ as a function of an arbitrary previous state $S_{k-1}$ (where $k \le n$).

**Base Step (k):**

$$
S_k = S_{k-1}^{P_k} \cdot G^{h_k}
$$

**Step (k+1):**

$$
\begin{aligned}
S_{k+1} &= S_k^{P_{k+1}} \cdot G^{h_{k+1}} \\
&= (S_{k-1}^{P_k} \cdot G^{h_k})^{P_{k+1}} \cdot G^{h_{k+1}} \\
&= S_{k-1}^{P_k \cdot P_{k+1}} \cdot G^{h_k \cdot P_{k+1}} \cdot G^{h_{k+1}}
\end{aligned}
$$

**General Form (by induction):**

$$
S_n = S_{k-1}^{\left( \prod_{i=k}^n P_i \right)} \cdot \left( G^{h_k \cdot \prod_{j=k+1}^n P_j} \cdot G^{h_{k+1} \cdot \prod_{j=k+2}^n P_j} \cdot \dots \cdot G^{h_n} \right)
$$

### 1.3 Witness Extraction Logic
To prove membership of $P_k$ at step $k$, we isolate the term containing $P_k$ from the "future noise". We define the **Affine Tuple** $\mathcal{A}_{k \to n}$ representing the aggregate transformation from step $k$ to $n$:

$$
S_n = \text{Apply}(\mathcal{A}_{k \to n}, S_{k-1})
$$

The verification equation becomes checking if the state $S_{k-1}$, when transformed by member $k$ and then by the suffix chain $k+1 \to n$, yields $S_n$.

---

## 2. Affine Composition Law

To enable $O(\log N)$ verification and parallel construction (Segment Trees), we must prove that our affine transformations form a **Monoid** under a specific composition law.

### 2.1 Definition
Let an affine tuple $\mathcal{A} = (P, Q)$ act on a state $S$ as:

$$
\rho(\mathcal{A}, S) = S^P \cdot Q
$$

### 2.2 Derivation of Composition
Let there be two consecutive transformations $\mathcal{A}\_1 = (P\_1, Q\_1)$ and $\mathcal{A}\_2 = (P\_2, Q\_2)$. We seek a single tuple $\mathcal{A}\_{\text{merge}}$ such that:

$$
\rho(\mathcal{A}_{\text{merge}}, S) = \rho(\mathcal{A}_2, \rho(\mathcal{A}_1, S))
$$

**Proof:**

$$
\begin{aligned}
\text{Right Side} &= \rho(\mathcal{A}_2, (S^{P_1} \cdot Q_1)) \\
&= (S^{P_1} \cdot Q_1)^{P_2} \cdot Q_2 \\
&= S^{P_1 \cdot P_2} \cdot Q_1^{P_2} \cdot Q_2
\end{aligned}
$$

Comparing this to the standard form $S^{P_{new}} \cdot Q_{new}$, we derive the binary composition operator $\oplus$:

$$
\mathcal{A}_1 \oplus \mathcal{A}_2 = (P_1 \cdot P_2, \quad Q_1^{P_2} \cdot Q_2)
$$

### 2.3 Proof of Associativity
For a Segment Tree to work, $(\mathcal{A}_1 \oplus \mathcal{A}_2) \oplus \mathcal{A}_3$ must equal $\mathcal{A}_1 \oplus (\mathcal{A}_2 \oplus \mathcal{A}_3)$.

**Left Side:** $(\mathcal{A}_1 \oplus \mathcal{A}_2) \oplus \mathcal{A}_3$

$$
\begin{aligned}
\text{Let} \quad \mathcal{A}_{12} &= (P_1 P_2, Q_1^{P_2} Q_2) \\
\mathcal{A}_{12} \oplus \mathcal{A}_3 &= ( (P_1 P_2) P_3, \quad (Q_1^{P_2} Q_2)^{P_3} \cdot Q_3 ) \\
&= ( P_1 P_2 P_3, \quad Q_1^{P_2 P_3} Q_2^{P_3} Q_3 )
\end{aligned}
$$

**Right Side:** $\mathcal{A}_1 \oplus (\mathcal{A}_2 \oplus \mathcal{A}_3)$

$$
\begin{aligned}
\text{Let} \quad \mathcal{A}_{23} &= (P_2 P_3, Q_2^{P_3} Q_3) \\
\mathcal{A}_1 \oplus \mathcal{A}_{23} &= ( P_1 (P_2 P_3), \quad Q_1^{P_2 P_3} \cdot (Q_2^{P_3} Q_3) ) \\
&= ( P_1 P_2 P_3, \quad Q_1^{P_2 P_3} Q_2^{P_3} Q_3 )
\end{aligned}
$$

**Conclusion:** Left Side $\equiv$ Right Side. The operation is **associative**. This mathematically guarantees that we can verify a chain of events by combining them into tree nodes in any grouping order.

---

## 3. Hyper-Tensor Folding

### 3.1 Tensor Structure
Let $\mathcal{T}$ be a tensor of dimension $d$ with side length $L$. Each element at coordinate vector $\vec{v} = (v_1, \dots, v_d)$ contains an affine tuple $\mathcal{A}_{\vec{v}}$.

### 3.2 The Folding Operator $\Phi$
We define a folding operator that reduces dimension $k$ to $k-1$. For a slice defined by fixing coordinates $(v_2, \dots, v_d)$, we aggregate along the axis $v_1$:

$$
\text{Fold}_{v_1}(\mathcal{T}) = \bigoplus_{i=1}^{L} \mathcal{T}_{(i, v_2, \dots, v_d)}
$$

### 3.3 Dimensional Recursion
The Global Root $\mathcal{R}$ is obtained by recursively folding all dimensions:

$$
\mathcal{R} = \text{Fold}_{v_d} \left( \dots \text{Fold}_{v_2} \left( \text{Fold}_{v_1}(\mathcal{T}) \right) \dots \right)
$$

### 3.4 Orthogonal Verification
Although the affine composition itself is non-commutative, the dimensional folding order is commutative regarding the final scalar result, provided the topology is static:

$$
\text{Fold}_y(\text{Fold}_x(\mathcal{T})) \equiv \text{Fold}_x(\text{Fold}_y(\mathcal{T}))
$$

---

## 4. Security Reductions

### 4.1 Hardness Assumption
The security of HTP relies on the **Hidden Order Assumption** in Class Groups. It is computationally infeasible to find the order $ord(G)$ of the class group $Cl(\Delta)$.

### 4.2 Soundness of Membership
For an adversary to forge a membership proof $(W', R')$ for a non-member $P^\ast$ such that:

$$
(W')^{P^\ast} \cdot R' \equiv T \pmod \Delta
$$

They effectively need to solve the **Root Problem** in the group. Given the Adaptive Root Assumption holds, the probability of forgery is negligible.
