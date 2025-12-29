# THEORY PATCH: The Unified Objective Function
## Bridging the Gap between Signal and Loss via Free Energy

### 1. The Schism
The system currently receives two disparate signals:
1.  **The Prior (Generator)**: $z_0 \in \mathbb{R}^V$. Goal: Stay close to the "natural" text distribution.
2.  **The Likelihood (STP)**: $E_{STP}(a) \in [0, \infty]$. Goal: Satisfy hard logical constraints.

Without a unified objective, the controller oscillates between "**Valid but Gibberish**" and "**Fluent but Wrong**".

---

### 2. Formal Definition: Variational Free Energy
We define the objective of the Bias Controller as minimizing the **Variational Free Energy $\mathcal{F}$** of the modified distribution $Q_{\vec{b}}$.

Let $P_{prior} = \text{Softmax}(z_0)$ be the generator's original probability.
Let $Q_{\vec{b}} = \text{Softmax}(z_0 + P\vec{b})$ be the biased distribution.
Let $E(a)$ be the STP Energy of action $a$.

The **Total Loss Function $\mathcal{L}(\vec{b})$** is defined as:
$$\mathcal{L}(\vec{b}) = \underbrace{\mathbb{E}_{a \sim Q_{\vec{b}}} [E(a)]}_{\text{Expected Logical Error}} + T \cdot \underbrace{D_{KL}(Q_{\vec{b}} || P_{prior})}_{\text{Semantic Drift}}$$

* **Term 1 (Accuracy)**: Forces the distribution towards logically valid actions ($E=0$).
* **Term 2 (Fidelity)**: Penalizes deviating too far from the LLM's original intent (the "Bias Cost").
* **$T$ (Temperature)**: The Lagrange multiplier controlling the trade-off.



---

### 3. The Optimization Hierarchy (Lexicographical)
In practice, logical validity is often non-negotiable. Thus, we define the optimization as **Lexicographical** in the limit $T \to 0$.
Find $\vec{b}^*$ such that:
1.  **Primary**: $\vec{b}^* \in \text{argmin}_{\vec{b}} \mathcal{E}_{STP}(\text{Dec}(\vec{b}))$
2.  **Secondary**: Among all valid $\vec{b}$, minimize $||\vec{b}||_{p-adic}$ (or equivalently, minimize $D_{KL}$).

---

### 4. VAPO's Role: Metropolis-Hastings Implementation
This formulation explains why VAPO works as a **Metropolis-Hastings sampler**.
The acceptance probability for a perturbation $\vec{b} \to \vec{b}'$ is:
$$\alpha = \min \left( 1, \frac{e^{-\mathcal{L}(\vec{b}')/T}}{e^{-\mathcal{L}(\vec{b})/T}} \right) = \min \left( 1, e^{-\frac{\Delta E + T \cdot \Delta D_{KL}}{T}} \right)$$

This confirms that VAPO is implicitly sampling from the **Posterior Distribution of Truth**:
$$P^*(a | \text{Logic}) \propto P_{prior}(a) \cdot e^{-E(a)/T}$$



---

### 5. Code Implication
The `BiasController` struct should track these two metrics separately to allow for dynamic temperature scheduling (**Annealing**).

```rust
struct ObjectiveMetrics {
    logical_energy: f64,   // E_STP
    semantic_drift: f64,   // KL(Q || P)
    total_loss: f64,       // E + T * KL
}
```

This ensures the controller knows what it is minimizing: it is minimizing the **Information Theoretic Cost** of correcting the model.
