# THEORY PATCH: The Closed-Loop Formalism

### Defining the Bilevel Optimization System

#### 1. The Composition Deficit

The system components are defined, but their coupling is loose.
If $z' = \Phi(z, S)$ and $Loss = \mathcal{L}(z')$, then calculating $\nabla_\theta \mathcal{L}$ requires differentiating through the optimization process $\Phi$.
Without defining $\Phi$'s mathematical properties, the gradient chain rule $\frac{\partial \mathcal{L}}{\partial z'} \frac{\partial z'}{\partial z} \frac{\partial z}{\partial \theta}$ is undefined.

#### 2. Formal Definition: Bilevel Problem

Evolver is formally a **Stochastic Bilevel Optimization System**.

**The Inner Problem (Inference Time)**

The Control Operator $\Phi$ is the solution map of the VAPO process:

$$\Phi(z, S) = z + P \cdot b^*$$

Where $b^*$ is a random variable sampled from the Boltzmann distribution of the Free Energy:

$$b^* \sim \frac{1}{Z} \exp\left(-\frac{\mathcal{E}_{STP}(S, \text{Dec}(z+Pb)) + \lambda ||b||}{T}\right)$$

**Properties of $\Phi$:**
* **Stochastic:** $\Phi(z, S)$ is a random variable (Markov Kernel).
* **Time-Variant:** Depends on $S$ (the history/context).
* **Non-Differentiable:** Due to the discrete nature of $b$ and the argmax decoder.
* **Measurable:** We assert $\Phi$ is Borel measurable to ensure expectations are well-defined.

**The Outer Problem (Training Time)**

The training objective is to minimize the expected loss over the distribution of controlled trajectories:

$$\min_\theta \mathbb{E}_{x, S} \mathbb{E}_{b \sim \Phi} [\mathcal{L}_{task}(\text{Dec}(z_\theta(x) + Pb)) + \beta ||b||]$$

* $\mathcal{L}_{task}$: Standard Cross-Entropy against ground truth (or self-consistency reward).
* $\beta ||b||$: The **Compliance Loss**. This is crucial. It penalizes the generator for requiring heavy correction.

#### 3. Closing the Gradient Loop

Since $\Phi$ is not differentiable, we cannot compute $\nabla_\theta \Phi$ directly. We define the closed-loop gradient estimation strategy via **Proxy Optimization (Target-Based Learning)**:

Instead of differentiating through VAPO, we treat the Aligned Output $z_{aligned} = \Phi(z_{raw}, S)$ as a **Target (Pseudo-Label)**.

The gradient update becomes:

$$\theta_{t+1} \leftarrow \theta_t - \eta \nabla_\theta D_{KL}(\text{Softmax}(z_{aligned}) || \text{Softmax}(z_\theta))$$

**Theorem (Closed Loop Stability):**
The loop is stable if the generator $\theta$ converges to a manifold where the required bias $||b^*|| \to 0$.
This transforms the Bias Controller from a "Crutch" (permanent fix) to a "Teacher" (temporary guide).

#### 4. Mathematical Object: The Evolution Operator

We define the total system evolution $\Psi_{sys}$ as a discrete dynamical system on the product space $\Theta \times S$:

$$\begin{cases}
S_{t+1} \sim \mathcal{R}(S_t, \text{Dec}(\Phi(G_\theta(S_t), S_t))) & \text{(State Dynamics)} \\
\theta_{t+1} \leftarrow \text{Optimizer}(\theta_t, \Phi, S_t) & \text{(Weight Dynamics)}
\end{cases}$$

This formalizes Evolver as an **Adaptive Control System** where the controller ($\Phi$) acts on the fast timescale (Inference) and the plant ($G_\theta$) adapts on the slow timescale (Training) to minimize control effort.

#### 5. Type System Update

The return type of the engine must reflect this bilevel nature.

```rust
struct ClosedLoopTrace {
    raw_logits: LogitSection,       // z_theta
    optimal_bias: BiasVector,       // b* (Inner Solution)
    aligned_logits: LogitSection,   // z_aligned
    gradient_target: LogitSection,  // Target for Backprop
}
```
