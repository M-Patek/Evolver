# THEORY PATCH: The Unified Objective Function

From Gradient Descent to Metropolis-Hastings Proposals

## 1. The Schism

The system must reconcile two forces:

* The Generator Prior ($P_{gen}$): Wants to produce "natural" text (Low Perplexity).
* The Logical Constraints ($E_{STP}$): Wants to satisfy algebraic truth (Zero Energy).

Since $E_{STP}$ is discrete and non-differentiable, we cannot use standard loss functions + backprop.

## 2. Formal Definition: Variational Free Energy

We frame the control problem as Sampling from a Posterior Distribution.
We want to sample actions $a$ from the "Truth Distribution" $P^*$:

$$
P^*(a | \text{Context}) \propto P_{gen}(a | \text{Context}) \cdot e^{-\beta E_{STP}(a)}
$$

Where $\beta$ is the inverse temperature (strictness of logic).

## 3. The Role of the Transformer: Learning the Proposal

Direct sampling from $P^*$ is intractable (requires exhaustive search).
We introduce a parameterized Proposal Distribution $Q_\theta(\vec{b} | \text{Context})$ (The Intuition Engine).

The objective of the Transformer ($\theta$) is to minimize the Inclusive KL Divergence (or minimize the Forward Amortized Inference Cost):

$$
\mathcal{L}(\theta) = \mathbb{E}_{\text{Context}} \left[ D_{KL}(P^* || Q_\theta) \right]
$$

In practice, this means Self-Imitation Learning:

* **Explore:** VAPO runs a computationally expensive search to find a valid $\vec{b}^*$ such that $E(\vec{b}^*) = 0$.
* **Train:** The Transformer updates $\theta$ to maximize the likelihood of guessing $\vec{b}^*$ in the future.

$$
\nabla_\theta \mathcal{L} \approx - \nabla_\theta \log Q_\theta(\vec{b}^* | \text{Context})
$$

## 4. The VAPO Algorithm (Metropolis-Hastings)

With the trained proposal $Q_\theta$, the runtime loop becomes efficient:

* **Propose:** Sample candidate bias $\vec{b}' \sim Q_\theta(\cdot | \text{Context})$.
* **Evaluate:** Calculate Energy $E(\vec{b}')$.
* **Accept/Reject:**
    * If $E(\vec{b}') = 0$: Accept (Instant solution).
    * If $E(\vec{b}') > 0$: Reject and perform local perturbation (random walk) starting from $\vec{b}'$.

## 5. Summary

* **Old View:** Transformer predicts gradients $\to$ Impossible on discrete steps.
* **New View:** Transformer predicts Proposals for a Metropolis-Hastings sampler.
* **Result:** The system "learns to search" faster over time, but correctness is always guaranteed by the verifier (STP), not the neural network.
