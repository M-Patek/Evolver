# Geometry of Nonsmooth Space: Stratification & Surrogate Gradients

## 1. The Stratified Manifold Hypothesis

The core premise of the Evolver system is that the space of logical thoughts is a Stratified Space (specifically, a Whitney Stratified Space).

$$\mathcal{M} = \bigcup_{i} S_i$$

Where:

* Each $S_i$ (stratum) is a smooth submanifold representing a fixed logical structure (e.g., "All proofs that use Modus Ponens").
* The transitions between strata are singular boundaries where dimensions change.

## 2. The Gradient Problem

In standard deep learning, we optimize on a smooth manifold $\mathbb{R}^n$. In Evolver, we optimize on $\mathcal{M}$.
The problem is that the map from Control Space ($\mathbb{R}^k$) to Logical Space ($\mathcal{M}$) passes through the Argmax discretization layer.

$$\vec{b} \xrightarrow{P} \mathbb{R}^V \xrightarrow{\text{Argmax}} \text{Discrete Action} \xrightarrow{\text{STP}} E$$

* **Inside a Stratum:** The discrete action is constant. 
$$\nabla_{\vec{b}} E = 0$$
* **At a Boundary:** The action jumps. 
$$\nabla_{\vec{b}} E \text{ is undefined (Dirac delta)}$$

## 3. The Solution: Surrogate Gradient via Embedding

We cannot use the gradient of the logical energy. Instead, we use the Gradient of the Embedding Energy as a proxy.

We define a Surrogate Function $\hat{J}(\vec{b})$ operating purely in the continuous embedding space:

$$\hat{J}(\vec{b}) = || \text{Embedding}(\text{Current}) - \text{Embedding}(\text{Target}) ||_2$$

This function $\hat{J}$ is smooth and convex (quadratic).

### 3.1 The "Proxy" Hypothesis

VAPO assumes that minimizing the Euclidean distance in the embedding space ($\hat{J}$) increases the probability of snapping to the correct logical stratum in the discrete space ($J$).

$$\nabla \hat{J} \approx \mathbb{E}[\text{Direction to better logic}]$$

This is why BiasProjector calculates residuals on vectors, not on symbolic states.

## 4. Clarke Tangent Cone & Retraction

To rigorously define "movement" across boundaries, we use Nonsmooth Analysis.

### 4.1 Clarke Tangent Cone

At a singular point $x$ (a logical boundary), the Clarke Tangent Cone $T_C(x)$ captures the set of directions that are "statistically safe" to traverse.
In VAPO, the Projection Matrix $P$ learns to align its column vectors with this cone.

### 4.2 Retraction Mapping

The Argmax operation acts as a Retraction $\mathcal{R}$:

$$\mathcal{R}: \mathbb{R}^V \to \mathcal{M}$$

It projects the continuous "intent" onto the nearest valid logical "stratum".

## 5. Conclusion: The Optimization Objective

The problem solved by the Evolver Sidecar is effectively:

$$\min_{\vec{b} \in \mathbb{R}^k} E(\mathcal{R}(\text{Logits} + P(\vec{b})))$$

The optimization trajectory does not follow a smooth geodesic. It follows a Stochastic Tunneling path:

* **Drift:** Follow the Surrogate Gradient ($\nabla \hat{J}$) to get close to the target stratum.
* **Snap:** Rely on noise/perturbation to jump over the Argmax threshold (Boundary Crossing).
* **Lock:** Once $E=0$, the high energy barrier of the valid stratum traps the system in the correct state.
