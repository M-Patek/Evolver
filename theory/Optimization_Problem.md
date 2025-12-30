THEORY PATCH: The Optimization Problem Statement

Formalizing VAPO as a Variable Neighborhood Search (VNS)

1. The Formulation

The Bias Controller solves a Black-Box Discrete Optimization problem at each time step $t$. We explicitly acknowledge that the objective function is non-differentiable and terraced.

2. Problem Variables

Decision Variable:
$\vec{b} \in \mathbb{Z}_L^k$
(Discrete Bias Vector on Torus).

Constants:

$z_0 \in \mathbb{R}^V$
: Base logits from Generator.

$P \in \mathbb{R}^{V \times k}$
: Projection matrix (The "Canon").

$s_t$
: Current STP algebraic state.

3. The Objective Function

The objective $J(\vec{b})$ is a composite Lagrangian with a Hard Logic Barrier:

$$J(\vec{b}) = \underbrace{\mathcal{E}_{STP}(s_t, \Pi(z_0 + P\vec{b}))}_{\text{Discrete Step Function}} + \lambda \cdot \underbrace{\mathcal{V}_{p}(\vec{b})}_{\text{Regularization}}$$

$\Pi$
: The Voronoi Decoder (Argmax).

$\mathcal{E}_{STP}$
: The Energy functional ($0$ if valid, $>0$ if invalid).

$\mathcal{V}_{p}$
: The p-adic Valuation Cost. We prefer "fine-tuning" (high valuation perturbations) over "structural changes" (low valuation perturbations).

4. Constraints

Hard Constraint (Logic): The search loop must eventually find $\vec{b}^*$ such that
$\mathcal{E}_{STP}(\vec{b}^*) = 0$
.

Domain Constraint:
$\vec{b} \in [0, L)^k$
(Torus boundary).

Budget Constraint:
$N_{iter} \le N_{max}$
(Real-time limit).

5. VAPO as a Solver Algorithm

Since Gradient Descent is inapplicable to $J(\vec{b})$ directly, VAPO implements a Variable Neighborhood Search (VNS) strategy, jumping between "Surrogate Gradient Descent" and "Random Tunneling".

Algorithm State: Current solution
$\vec{b}_{curr}$
.

Modes of Operation:

Surrogate Descent (Fine-Tuning):

Assumption: Locally, the embedding space gradient approximates the logical gradient.

Action: b_new = b_curr - alpha * project(residual)

Effective when:
$E > 0$
but small (near the correct semantic cluster).

Stochastic Tunneling (Coarse Jump):

Assumption: Stuck in a local minimum (Logic Trap).

Action: Apply a perturbation to Low-Valuation bits (Large $p$-adic distance).

Effective when:
$E$
is high or stalled.

6. Code Implication: Explicit Cost Function

The Rust implementation separates the "Cost Evaluator" from the "Search Strategy".

```rust
struct OptimizationProblem<'a> {
    base_logits: &'a [f64],
    stp_ctx: &'a STPContext,
}

impl<'a> OptimizationProblem<'a> {
    /// Evaluates J(b). Note: This step function has zero gradient usually.
    fn evaluate(&self, bias: &BiasVector) -> f64 {
        let action = self.decode(bias);
        let energy = self.stp_ctx.energy(&action);
        let regularization = bias.p_adic_norm();
        energy + LAMBDA * regularization
    }
}
```
