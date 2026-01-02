# Constraint Semantics & Trust Model

## 1. Core Shift: From "Correct-by-Construction" to "Formally Verifiable"

In the early designs of Evolver, we utilized the term "Correct-by-Construction." To pursue absolute mathematical rigor, we have now refined this to "Formally Verifiable Search."

Evolver does not promise that a "solution will definitely be found." Instead, it promises that "if a solution is found, it must strictly adhere to the laws of physics, and this conclusion can be verified by a third party at a low cost."

We divide the system's trust boundaries into three levels:

| Level | Attribute | Responsible Party | Guarantee |
| :--- | :--- | :--- | :--- |
| **L0** | **Soundness** | Soul (STP Kernel) | If $x_{t+1} = M \ltimes x_t$, then $x_{t+1}$ strictly satisfies defined logical constraints. |
| **L1** | **Verifiability** | Tracer | Given initial state $x_0$ and action sequence $u_{0:k}$, anyone can replicate the evolution path. |
| **L2** | **Completeness** | Will (Optimizer) | No guarantee. The optimizer makes a best-effort attempt to find a path where $E \to 0$, but may fall into local optima. |

---

## 2. Level 0: Logical Soundness via STP

The core of Evolver lies in leveraging the **Semi-Tensor Product (STP)** to transform logical constraints into algebraic equations.

For any logical constraint $L(x) = \text{True}$, we encode it into the algebraic form $L \ltimes x = \delta_n^1$. The system's dynamical equation $x_{t+1} = f(x_t, u_t)$ is constructed as:

$$x_{t+1} = M \ltimes u_t \ltimes x_t$$

where the structural matrix $M$ is the algebraic realization of the Logical AND of all local constraints.

**Theorem (Soundness):** If the structural matrix $M$ is generated through canonical STP logical transformations, then for any time $t$, as long as the evolution follows the dynamical equations, the state $x_t$ cannot violate any hard constraints.

This implies that the system is fundamentally incapable of expressing a state that violates physical laws. Erroneous states do not exist within the algebraic structure, or they are mapped to the null space.

---

## 3. Level 1: Search Verifiability

While STP ensures the legality of single-step evolution, it does not guarantee long-term goal achievement. This necessitates the concept of "verification."

The system output is not a single result, but a **Proof of Will (Search Proof)**, containing:
* Initial state $x_0$
* Sequence of perturbations (actions) $\mathcal{U} = \{u_0, u_1, \dots, u_k\}$
* Claimed final energy $E_{claim}$

### Verification Protocol
The Verifier does not need to run complex optimization algorithms; they only need to perform a deterministic **Replay**:

```rust
fn verify(proof: Proof) -> bool {
    let mut x = proof.initial_state;
    for u in proof.action_sequence {
        // STP matrix multiplication: highly deterministic, low computational cost
        x = M * u * x; 
    }
    return energy(x) == proof.claimed_energy;
}
```

This asymmetric computational cost (extremely expensive to search, extremely cheap to verify) is the core value of the Evolver architecture.

---

## 4. Level 2: The Limit of Will

Will (Optimizer) is a heuristic searcher operating in complex topological spaces (based on v-PuNNs strategies).

We must honestly state that:
1.  Optimization problems are generally non-convex.
2.  There exist numerous local minima.

Therefore, Evolver cannot guarantee finding the global optimum where $E=0$. When the system outputs a non-zero energy state, it represents the "best fit found under limited computational resources," rather than an absolute truth.

## Conclusion

Evolver does not provide an "Oracle," but rather a "Mathematically Rigorous Explorer." It will not lie (L0 & L1), but it may fail (L2) 。
