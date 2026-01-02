# IDEAL MODEL SPECIFICATION: The VAPO Implementation
"Detailed architectural blueprint for the Soul, Will, and Body implementation."

## 1. Overview
This document specifies the ideal model implementation details for each component within the Evolver system, directly corresponding to the code structure under `src/`. The system aims to solve logic generation problems through algebraic evolution.

## 2. Soul Layer: `src/soul/`

### 2.1 Algebraic Engine (`algebra.rs`)
* **Core Struct:** `ClassGroupElement { a: BigInt, b: BigInt, c: BigInt }`
* **Operations:** Implements **NUCOMP** or **NUDUPL** algorithms for efficient Gaussian composition.
* **Invariant:** All `compose()` operations must automatically trigger `reduce()` to ensure the state remains in Reduced Form at all times.

### 2.2 Dynamical Systems (`dynamics.rs`)
* **Orbit:** Defines the system's trajectory across the group.
* **Hash Chain:** Trajectories must record not only the position but also the path hash $H_{t} = \text{Hash}(H_{t-1} || S_t)$ to prevent path tampering.

## 3. Will Layer: `src/will/`

### 3.1 Perturber (`perturber.rs`)
The perturber maintains a hierarchical pool of prime ideals:
* **Layer 1 (Fine):** Small-norm primes (e.g., $p < 100$). Used for local gradient descent simulation.
* **Layer 2 (Coarse):** Medium-norm primes. Used to escape shallow local minima.
* **Layer 3 (Chaos):** Large-norm primes or random elements. Used for "resets" or drastic tunneling effects.

### 3.2 Optimizer Core (`optimizer.rs`)
Implements the **VAPO State Machine**:
* **Valuation:** Uses `evaluator.rs` to compute the energy of the current state.
* **Adaptation:** If energy fails to decrease for $N$ consecutive steps (Stagnation), the perturbation level is automatically escalated.

## 4. Body Layer: `src/body/`

### 4.1 The Dual Projection Architecture
To satisfy both "Searchability" and "Security," the Body must implement two orthogonal projection functions:

**A. Heuristic Projection $\Psi_{topo}$ (`projection.rs` - `lipschitz_map`)**
* **Purpose:** To provide geometric guidance for the Will layer.
* **Implementation:** Based on Modular Forms or simple coefficient modulo buckets.
* **Property:** **Lipschitz Continuity.** A tiny algebraic change $\delta S$ in the input results in a proportional change $\delta O$ in the output. This enables VAPO to perceive "direction."

**B. Exact Projection $\Psi_{exact}$ (`projection.rs` - `cryptographic_map`)**
* **Purpose:** To generate the final code/logic and prevent forgery.
* **Implementation:** $O = \text{SHA3-512}(\text{Canonical}(S))$.
* **Property:** **Avalanche Effect.** A tiny algebraic change leads to a complete transformation of the output.
* **Adapter (`adapter.rs`):** Deterministically converts the hash stream into a sequence of `ProofAction` (e.g., opcodes, AST nodes).

## 5. DSL Layer: `src/dsl/`

### 5.1 STP Bridge (`stp_bridge.rs`)
* **Compilation:** Compiles natural language or logical constraints into a structure matrix $M$.
* **Energy Calculation:**
$$E = \| M \ltimes (\ltimes x_i) - y \|^2$$
Where $M$ is the matrix representation of logical rules, $x_i$ represents variables projected from the state, and $y$ is the target output.

---

## Summary
This implementation specification ensures that Evolver is more than just a random searcher. By leveraging the Soul for structure, the Body (Lipschitz projection) for gradients, and the Will (VAPO) for strategy, the system can efficiently converge to "Truth" within discrete algebraic spaces.
