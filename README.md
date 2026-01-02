# Evolver: The Algebraic Logic Generator

> "Logic is not corrected; it is evolved."

Evolver is a native algebraic logic generation engine based on the **Semi-Tensor Product (STP)** and **Ideal Class Groups of Imaginary Quadratic Fields**.

Unlike traditional "black-box" Large Language Models (LLMs), Evolver does not predict probabilities. Instead, it directly "grows" truth paths that satisfy logical constraints through **Valuation-Adaptive Perturbation Optimization (VAPO)** searches within rigorous algebraic structures.

We do not promise "Correct-by-Construction"; we offer a higher-level guarantee — **Verified-by-Search**.

---

## Core Philosophy: The Trinity

Evolver's architecture mimics the forms of life, decoupling the system into three orthogonal dimensions (corresponding to the code structure):

### 1. Soul: Algebraic Laws (`src/soul`)
* **Mathematical Entity**: Ideal Class Group $Cl(\Delta)$ of an imaginary quadratic field.
* **Role**: Defines the physical laws of the system. Regardless of how the state evolves, it remains a valid group element, ensuring **Algebraic Soundness**.
* **Characteristics**: A discrete, vast Cayley graph with deterministic chaotic features.

### 2. Will: Evolutionary Dynamics (`src/will`)
* **Core Algorithm**: VAPO (Valuation-Adaptive Perturbation Optimization).
* **Role**: Drives the system's movement across the group manifold, replacing traditional gradient descent.
* **Structural Jumps**: Uses prime ideals with large norms for coarse-grained searching.
* **Fine-tuning**: Uses prime ideals with small norms for local optimization.
* **Objective**: To find states where the unified energy $J(S) \to 0$.

### 3. Body: Topological Manifestation (`src/body`)
* **Mechanism**: Dual Projection.
    * $\Psi_{topo}$ (Lipschitz): Provides geometric intuition and a sense of "gradient" for the Will.
    * $\Psi_{exact}$ (Hash): Collapses the algebraic state into a unique, unforgeable code/logic path.
* **Result**: Even minute logical differences correspond to distinct geometric locations at the algebraic source.

---

## Quick Start

### Prerequisites
Evolver is a high-performance Rust project.

```bash
rustc --version  # Requires 1.70+
```

### Build
```bash
cargo build --release
```

### Example: Evolving a Logic XOR Gate
```rust
use evolver::prelude::*;

fn main() {
    // 1. Define the Body: A boolean network with 2 inputs and 1 output
    let topology = Topology::new(2, 1);

    // 2. Define the Soul: Compile logical constraints (STP)
    // Constraint: Output is 1 when inputs differ; otherwise 0
    let constraints = StpBridge::compile("y = x1 XOR x2");

    // 3. Inject the Will: Configure the VAPO optimizer
    let optimizer = Optimizer::new()
        .strategy(Strategy::ValuationAdaptive)
        .max_epochs(1000)
        .target(Energy::Zero);

    println!("Evolving logic from algebraic void...");

    // 4. Begin Evolution
    match optimizer.evolve(topology, constraints) {
        EvolutionResult::VerifiedSuccess(trace) => {
            println!("✨ Truth path discovered!");
            println!("Algebraic Seed: {:?}", trace.seed);
            println!("Proof Hash: {}", trace.proof_hash);
        },
        EvolutionResult::ValidFailure(trace, energy) => {
            println!("⚠️ Trapped in local optimum (E={:.4})", energy);
            println!("Certificate of Cognitive Dissonance: {:?}", trace.compromise);
        }
    }
}
```

---

## Theoretical Foundation

Evolver’s core engine is built upon the following mathematical pillars:

* **Semi-Tensor Product (STP)**: Transforms logical operations into matrix multiplications, enabling the algebraization of logic.
* **Algebraic Number Theory**: Leverages the hardness of the discrete logarithm problem in class groups as the state space.
* **Topological Dynamics**: Utilizes the geometric properties of manifolds to guide discrete searches.

For detailed derivations, please refer to the whitepaper in the `theory/` directory.

---

## License

**M-Patek PROPRIETARY LICENSE**

Copyright © 2025 M-Patek. All Rights Reserved.

This software contains trade secrets protected by law. Unauthorized copying, distribution, or commercial use is strictly prohibited.
