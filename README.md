# Evolver: Neuro-Symbolic Alignment Orchestrator

(Bias-Controlled HTP Architecture)

> "Logic is not generated; it is orchestrated. Search is not blind; it is guided."

Evolver is a neuro-symbolic alignment system based on the Hyper-Tensor Protocol (HTP) and Semi-Tensor Product (STP). It solves the "Logical Hallucination" problem not by training a perfect generator, but by wrapping a chaotic LLM in a rigorous algebraic control loop.

## 🏛️ Core Architecture: The Four Pillars

The system is a closed loop involving four distinct components, transitioning from "probabilistic guessing" to "algebraic truth":

### 1. The Generator (Chaotic Core)

* **Role:** The "Dreamer". Provides raw cognitive primitives (Logits).
* **Traits:** Retains chaotic weights based on the Hidden Order Assumption.
* **Status:** Permitted to hallucinate.

### 2. The STP Engine (Constraint Checker)

* **Code:** `src/dsl/stp_bridge.rs`
* **Role:** The "Physics Engine".
* **Principle:** Maps logical actions to matrix operations.

$$
x(t+1) = L \ltimes x(t)
$$

* **Output:** Discrete Energy ($E$).

$$
E = 0.0
$$

: Logical Consistency (QED).

$$
E > 0.0
$$

: Violation (e.g., "Odd + Odd = Odd").

### 3. The Intuition Engine (Transformer Sidecar)

* **Role:** The "Navigator".
* **Type:** A lightweight Transformer trained on successful VAPO traces.
* **Function:** Proposal Generation.
* It does NOT provide gradients (the landscape is discrete).
* It provides a Proposal Distribution.

$$
Q(\vec{b} | \text{Context})
$$

"Based on history, the solution is likely in this direction."

### 4. The Bias Controller (VAPO Optimizer)

* **Code:** `src/control/bias_channel.rs`
* **Role:** The "Driver".
* **Algorithm:** VAPO (Valuation-Adaptive Perturbation Optimization).
* **Mechanism:** Metropolis-Hastings Search.
* Takes the Proposal from the Intuition Engine.
* Performs local discrete search/perturbation to latch onto the exact $E=0$ state.
* Projects the final Bias Vector ($\vec{b}$) onto the Generator's logits.

## 🛠️ Implementation & Tech Stack

Built on Rust for type safety and zero-cost abstractions.

### The Control Loop (Revised)

* **Generate:** LLM outputs raw logits $z$.
* **Verify:** STP Engine calculates Energy $E(z)$. If $E=0$, emit.
* **Propose:** If $E>0$, Intuition Engine predicts a target region $\vec{b}_{init}$.
* **Search:** VAPO performs fine-grained discrete search around $\vec{b}_{init}$ to find $\vec{b}^*$.
* **Project:**

$$
z_{final} = z + W \cdot \vec{b}^*
$$

* **Learn:** The trajectory is saved to train the Intuition Engine.

$$
(\text{Context}, \vec{b}^*)
$$

## 🚀 Quick Start

### Dependencies

* Rust (1.70+)
* `serde`, `rand`, `num-integer`

### Run Demo

The main program (`src/main.rs`) simulates the "Proposer-Optimizer" dynamic:

```bash
cargo run
```

### Expected Output

```text
🐱 New Evolver System Initializing...
--------------------------------------------------
[Init] STP Context loaded.
[Init] Intuition Engine (Mock) loaded.

📝 Mission: Prove that the sum of two Odd numbers is Even.
...
⚠️  [Step 3] Generating inference step...
   -> Raw Generator intent: Define 'sum' as Odd.
   -> STP Check: VIOLATION (Energy > 0).

🧠 [Intuition] Transformer proposing bias region: Sector 4.
🛡️ [VAPO] Optimizing local perturbation...
   -> Found correction vector [-1, 0, 1...]
   -> Energy dropped to 0.0.

✅ [Result] Action Corrected: Define 'sum' as Even.
```

## 📜 License

M-Patek PROPRIETARY LICENSE Copyright © 2025 M-Patek.
