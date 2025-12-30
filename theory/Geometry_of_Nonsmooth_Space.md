# Geometry of Nonsmooth Space: Stratification & Generalized Curvature

## 1. The Stratified Manifold Hypothesis

The core premise of the Evolver system is that the space of logical thoughts is not a smooth manifold, but a Stratified Space (specifically, a Whitney Stratified Space).

$$\mathcal{M} = \bigcup_{i} S_i$$

Where:

* Each $S_i$ (stratum) is a smooth submanifold representing a fixed logical structure (e.g., "All proofs that use Modus Ponens").
* The transitions between strata (logical jumps) are singular points where dimensions may change, and gradients are discontinuous.

### 1.1 The Impossibility of Direct Addition

In standard Deep Learning, we assume $\theta_{new} = \theta + \Delta \theta$.
In a Stratified Space, if $x \in S_i$ and we add a perturbation $\delta$, $x + \delta$ might fall into "the void" (invalid logic) or strictly cross into another stratum $S_j$.

Therefore, simple vector addition is undefined.

## 2. Clarke Tangent Cone & Retraction

To rigorously define "movement" in this space without violating logical constraints, we use Nonsmooth Analysis.

### 2.1 Clarke Tangent Cone

At a singular point $x$ (e.g., a logical conflict), the tangent space is not a flat plane. We define the Clarke Tangent Cone, $T_C(x)$, which captures all "admissible" directions that allow stabilization.

$$T_C(x) = \{ v \mid \forall x_n \to x, t_n \downarrow 0, \exists v_n \to v \text{ s.t. } x_n + t_n v_n \in \mathcal{M} \}$$

**Engineering Meaning:** This cone defines the set of all valid bias perturbations $\vec{b}$ that can locally resolve a conflict without breaking the STP constraints.

### 2.2 Retraction Mapping

Since we operate in the embedding space (Logits $\mathbb{R}^n$), we need a way to project our linear perturbations back onto the stratified manifold. We define a Retraction $\mathcal{R}$:

$$\mathcal{R}_x: T_x \mathcal{M} \to \mathcal{M}$$

In Evolver, this is implemented by the STP Decoder.

$$\text{Decode}(L + W \cdot \vec{b}) \approx \mathcal{R}(x + \vec{b})$$

## 3. Generalized Curvature (The "Hessian" Correction)

**Correction Note:** In previous heuristic descriptions, we referred to the "Hessian" of the energy function. However, since $E(x)$ is Lipschitz continuous but not $C^2$ (due to logical discrete jumps), the standard Hessian does not exist at singular points. We hereby introduce the rigorous formulation.

### 3.1 The Generalized Hessian (Clarke)

For the locally Lipschitz energy function $E: \mathbb{R}^n \to \mathbb{R}$, we define the Generalized Hessian, denoted as $\partial^2 E(x)$, as the convex hull of the limits of standard Hessians evaluated at nearby differentiable points.

Let $\Omega_E$ be the set of points where $E$ is twice differentiable (by Rademacher's theorem, this is almost everywhere). Then:

$$\partial^2 E(x) = \text{co} \left\lbrace \lim_{i \to \infty} \nabla^2 E(x_i) \mid x_i \to x, x_i \in \Omega_E \right\rbrace$$

This set $\partial^2 E(x)$ contains a family of matrices, not a single matrix.

### 3.2 Sigma-Curvature ($\Sigma$-Curvature)

We define the "Curvature" of a logical state not as a single number, but as the maximal spectral radius within the Generalized Hessian. We call this the $\Sigma$-Curvature:

$$\kappa_{\Sigma}(x) = \sup_{H \in \partial^2 E(x)} \| H \|_{op}$$

### 3.3 Physical Interpretation in VAPO

This rigorous definition aligns perfectly with the behavior of the Bias Controller:

* **Inside a Stratum (Smooth Logic):**
    * $x$ is in a smooth region $S_i$. $\partial^2 E(x)$ collapses to the singleton $\{ \nabla^2 E(x) \}$.
    * $\kappa_{\Sigma}$ is small.
    * **VAPO Behavior:** Standard gradient descent works. The bias vector $\vec{b}$ evolves smoothly.

* **At a Boundary (Logical Conflict/Jump):**
    * $x$ is at a singularity (crossing from "Even" to "Odd"). The sequence of Hessians from different sides approaches different limits.
    * $\partial^2 E(x)$ is a large set containing "infinite" or very steep distinct matrices.
    * $\kappa_{\Sigma}$ spikes to infinity.
    * **VAPO Behavior:** The "Energy Gradient" becomes unstable (subdifferential is large). The controller detects this Second-Order Variation and switches from "Optimization" to "Search/Jump" mode.

## 4. Conclusion: The Optimization Objective

The problem solved by the Evolver Sidecar is strictly defined as:

$$\min_{\vec{b} \in \mathbb{R}^k} E(\mathcal{R}(\text{Logits} + P(\vec{b})))$$

Where:

* $E$ is the STP Energy (Lipschitz, Non-smooth).
* $\mathcal{R}$ is the Retraction (Decoder).

The optimization trajectory follows the path of minimal generalized curvature to ensure stability.

By treating the system as a Nonsmooth Dynamic System, we justify why the Bias Controller requires perturbation (exploring the Clarke Cone) rather than just differentiation (which fails at singularities).
