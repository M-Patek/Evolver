# Geometry of Nonsmooth Space: Stratification & Generalized Curvature

## 1. The Stratified Manifold Hypothesis

The core premise of the Evolver system is that the space of logical thoughts is not a smooth manifold, but a Stratified Space (specifically, a Whitney Stratified Space).

$$\mathcal{M} = \bigcup_{i} S_i$$

Where:

* Each $S_i$ (stratum) is a smooth submanifold representing a fixed logical structure (e.g., "All proofs that use Modus Ponens").
* The transitions between strata (logical jumps) are singular boundaries where dimensions may change, and gradients are typically discontinuous.

### 1.1 Lack of Closure under Linear Addition

In the ambient embedding space $\mathbb{R}^n$ (Logits), the vector addition $x_{new} = x + \delta$ is mathematically well-defined. However, the logical manifold $\mathcal{M}$ is not closed under this linear operation.

If $x$ lies on a valid logical stratum $S_i$ (e.g., "Integer Space"), a naive linear perturbation $\delta$ will almost surely move the point off the manifold:

$$x + \delta \notin \mathcal{M}$$

This is not an issue of the operation being "undefined," but rather a Feasibility Violation. The point $x + \delta$ exists in the embedding space but loses its logical semantics (falling into "the void" of invalid logic).

Therefore, we cannot treat optimization as a simple trajectory in vector space. Instead, we must treat it as a Retraction-based Optimization problem on a Riemann-embedded submanifold.

## 2. Clarke Tangent Cone & Retraction

To rigorously define "movement" in this space without violating logical constraints, we use Nonsmooth Analysis to characterize valid directions and projections.

### 2.1 Clarke Tangent Cone

At a singular point $x$ (e.g., a logical conflict boundary), the tangent space is not a flat plane. We define the Clarke Tangent Cone, $T_C(x)$, which captures all "admissible" directions that allow stabilization or valid transition.

$$T_C(x) = \{ v \mid \forall x_n \to x, t_n \downarrow 0, \exists v_n \to v \text{ s.t. } x_n + t_n v_n \in \mathcal{M} \}$$

**Engineering Meaning:** This cone defines the set of all valid bias perturbations $\vec{b}$ that can locally resolve a conflict without effectively breaking the STP constraints (or moving to a valid adjacent stratum).

### 2.2 Retraction Mapping

Since we operate in the embedding space (Logits $\mathbb{R}^n$), we need a way to project our linear perturbations back onto the stratified manifold. We define a Retraction $\mathcal{R}$:

$$\mathcal{R}_x: T_x \mathcal{M} \to \mathcal{M}$$

In Evolver, this is physically implemented by the STP Decoder.

$$\text{Decode}(L + W \cdot \vec{b}) \approx \mathcal{R}(x + \vec{b})$$

## 3. Generalized Curvature (The "Hessian" Correction)

**Theoretical Correction:** Previous iterations relied on Rademacher's theorem, which is insufficient for defining second-order curvature. Here, we adopt the Piecewise $C^2$ Assumption.

### 3.1 Stratified Regularity Assumption

We assume the Energy Function $E: \mathbb{R}^n \to \mathbb{R}$ induced by the STP constraints is Piecewise $C^2$ relative to the stratification $\mathcal{S} = \{S_i\}$.

* For each stratum $S_i$, the restriction $E|_{S_i}$ is $C^2$ smooth.
* The set of non-smooth points $\mathcal{K} = \mathbb{R}^n \setminus \bigcup \text{int}(S_i)$ has measure zero.

Let $\Omega_E = \bigcup_{i} \text{int}(S_i)$ be the domain where the standard Hessian $\nabla^2 E(x)$ is well-defined.

### 3.2 The Generalized Hessian (Clarke)

At a singular point $x \in \mathcal{K}$ (a logical conflict or boundary), the standard Hessian does not exist. We define the Generalized Hessian $\partial^2 E(x)$ as the convex hull of the limits of standard Hessians evaluated from nearby smooth strata:

$$\partial^2 E(x) = \text{co} \left\lbrace \lim_{k \to \infty} \nabla^2 E(x_k) \mid x_k \to x, x_k \in \Omega_E \right\rbrace$$

This set $\partial^2 E(x)$ contains a family of matrices representing the curvature behavior as one approaches the conflict from different logical preconditions.

### 3.3 Sigma-Curvature ($\Sigma$-Curvature)

We define the "Curvature" of a logical state not as a single number, but as the maximal spectral radius within the Generalized Hessian set. We call this the $\Sigma$-Curvature:

$$\kappa_{\Sigma}(x) = \sup_{H \in \partial^2 E(x)} \| H \|_{op}$$

### 3.4 Physical Interpretation in VAPO

This rigorous definition aligns perfectly with the behavior of the Bias Controller:

**Inside a Stratum (Smooth Logic):**

* $x \in S_i$. The generalized Hessian collapses to a singleton:
$$\partial^2 E(x) = \{ \nabla^2 E(x) \}$$
* $\kappa_{\Sigma}$ is finite and small.
* **VAPO Behavior:** The controller operates in "Fine-Tuning" mode (Small $p$-adic perturbations).

**At a Boundary (Logical Conflict):**

* $x$ is at a singularity. Approaching $x$ from the "Valid" side vs. the "Invalid" side yields drastically different Hessian limits.
* $\partial^2 E(x)$ becomes a large set.
* $\kappa_{\Sigma}$ spikes significantly (potentially $\to \infty$ for hard constraints).
* **VAPO Behavior:** The controller detects this Second-Order Instability and switches to "Coarse Search" mode (Large $p$-adic perturbations) to jump out of the high-curvature trap.

## 4. Conclusion: The Optimization Objective

The problem solved by the Evolver Sidecar is strictly defined as:

$$\min_{\vec{b} \in \mathbb{R}^k} E(\mathcal{R}(\text{Logits} + P(\vec{b})))$$

Where:

* $E$ is the STP Energy (Piecewise Smooth).
* $\mathcal{R}$ is the Retraction (Decoder).

The optimization trajectory follows the path of minimal generalized curvature to ensure logical stability. By treating the system as a Nonsmooth Dynamic System on a Stratified Manifold, we justify the necessity of the VAPO algorithm over standard gradient descent.
