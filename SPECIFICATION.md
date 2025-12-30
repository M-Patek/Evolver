# HYPER-TENSOR PROTOCOL (HTP): Technical Specification

## 1. Mathematical Preliminaries

### 1.1 Class Group Parameters

**Discriminant Generation:**
Define $\Delta = -M$, where $M$ is defined as:

$$
M \equiv 3 \pmod 4
$$

is a prime generated via Hash-to-Prime.

**Security:** Relies on the difficulty of computing the class number $h(\Delta)$.

### 1.2 Dual-Operator System

HTP utilizes two distinct algebraic operators to separate temporal causality from spatial topology.

**Time Operator ($\oplus_{\text{time}}$):** Non-commutative affine composition for history.

$$
\mathcal{A}_1 \oplus \mathcal{A}_2 = (P_1 P_2, \quad Q_1^{P_2} Q_2)
$$

**Space Operator ($\otimes_{\text{space}}$):** Commutative group aggregation for topology.

$$
\mathcal{A}_1 \otimes \mathcal{A}_2 = (P_1 P_2, \quad Q_1 Q_2)
$$

## 2. Affine Structure & Optimization

### 2.1 The Affine Tuple

Define the tuple $\mathcal{A} = (P, Q)$ where:

$$
P \in \mathbb{Z}
$$

$$
Q \in Cl(\Delta)
$$

### 2.2 Time Evolution (Neuron Memory)

Used within HTPNeuron memory cells to record sequential events.

* **Input:** Stream of affine tuples.
* **Aggregation:** Segment Tree using $\oplus_{\text{time}}$.
* **Result:** A single tuple $\mathcal{A}_{\text{cell}}$ representing the entire causal history of that memory cell.

### 2.3 Space Aggregation (Hyper-Tensor)

Used by the HyperTensor to fold dimensions.

* **Input:** Spatial grid of $\mathcal{A}_{\text{cell}}$ (snapshots).
* **Aggregation:** Dimensional folding using $\otimes_{\text{space}}$.
* **Result:** A unique Global Root that is independent of the folding order.

## 3. Hyper-Tensor Topology

### 3.1 Coordinate Mapping

Define the mapping from logical index $i$ to vector $\vec{v}$:

$$
v_k = (i // L^{k-1}) \pmod L
$$

### 3.2 Dimensional Folding

The tensor dimensionality reduction function $\Phi$ utilizes the Space Operator:

$$
\Phi(Tensor_d) = \bigotimes_{i=1}^{L} Tensor_{(i, \dots)}
$$

### 3.3 Orthogonal Anchoring

A valid proof for point $\vec{v}$ consists of a hybrid path:

* **Time Path (Micro):** The non-commutative Merkle path inside the cell at $\vec{v}$, verifying the specific event history.
* **Space Path (Macro):** The commutative Merkle path through the tensor dimensions along the Challenge Axis.

**Consistency Check:**
Since $\otimes_{\text{space}}$ is commutative, the Verifier can request folding along any axis (e.g., Y-axis), and the result must match the Global Root.

$$
\text{Fold}_{\text{challenge\_axis}}(\text{Slice}) \equiv \text{GlobalRoot}
$$

## 4. Protocol Flow & Verifiable Binding (UPDATED)

### 4.1 Security Assumption: The Splicing Gap

Previous versions allowed a "Splicing Attack" where a Bias Vector $\vec{b}$ generated for Context A could be replayed in Context B if the Verifier could not inspect the raw Generator output.
**v0.2 Fix:** We introduce Seed Commitment and Context Hashing.

### 4.2 The Proof Bundle Structure

A valid HTP Proof must strictly contain:

$$
\text{ProofBundle} := \{ 
  \vec{b}_{ctrl},       \quad // \text{The Bias Vector (Solution)}
  \text{Proof}_{alg},   \quad // \text{The STP Validity Proof}
  \mathbf{Seed}_{gen},  \quad // \text{Commitment to Chaos (New)}
  \mathbf{Hash}_{ctx}   \quad // \text{Binding to Input (New)}
\}
$$

### 4.3 Verification Algorithm (Deterministic Replay)

Verifier client logic:

**Context Binding Check:**
Compute local hash:

$$
h_{local} = \text{SHA256}(\text{ContextString})
$$

Assert:

$$
h_{local} == \text{ProofBundle.Hash}_{ctx}
$$

(Prevents replaying a proof in a wrong conversation)

**Generator Integrity Check (The Fix):**
Initialize a local, lightweight Generator model (or mock oracle) using $\text{ProofBundle.Seed}_{gen}$.
Run inference:

$$
\vec{z}_{raw} = \text{Generator}(\text{ContextString})
$$

(Ensures the "Problem" being solved is authentic)

**Algebraic Projection:**
Compute the projection of the bias vector:

$$
\vec{z}_{bias} = W_{proj}(\text{Seed}_{gen}) \cdot \vec{b}_{ctrl}
$$

(Note: The Projection Matrix $W$ must also be seeded deterministically!)

**Decision Verification:**
Compute effective logits:

$$
\vec{z}_{final} = \vec{z}_{raw} + \vec{z}_{bias}
$$

Assert:

$$
\text{Argmax}(\vec{z}_{final}) == \text{ClaimedAction}
$$

**Energy Check (STP):**
Assert:

$$
STP\_Energy(\text{ClaimedAction}) == 0
$$

This ensures that the Bias $\vec{b}$ is not just "a valid key", but "the specific key for this specific lock"
