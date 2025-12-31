# Energy Definition: The Discrete Logic Potential

### 1. Nature of Energy
In Evolver, energy $E$ is no longer a continuously differentiable Lyapunov function, but a **Discrete Logic Potential**. It is a binary or integer measure of the "degree of logical violation," directly reflecting the state of the STP (Semi-Tensor Product) engine, Master Meow (主人喵).

### 2. Formal Definition
The energy function 
$E: \mathcal{A}^* \to \{0, 1, \dots, \infty\}$
is defined over the sequence of logical actions $\mathcal{A}^*$.

For a generated proof path $\tau = (a_1, a_2, \dots, a_k)$:
$$E(\tau) = \sum_{t=1}^k \text{Violation}(S_{t-1}, a_t)$$

Where $\text{Violation}(S, a)$ is the local logic check function:
$$\text{Violation}(S, a) = \begin{cases} 0.0 & \text{if } a \text{ is valid in context } S \\ 1.0 & \text{if } a \text{ contradicts } S \text{ or axioms} \\ \text{undefined} & \text{if } a \text{ is syntactically malformed} \end{cases}$$

**Code Mapping:** `src/dsl/stp_bridge.rs`: The `calculate_energy` function directly returns 0.0 or 1.0.

### 3. Energy Landscape
Since energy is discrete (typically a superposition of 0s and 1s), the optimization landscape appears **Terraced** or **Plateau-like**.

* **No Gradients:** In any flat region, 
    $\nabla E = 0$.
* **Cliffs:** Transitions between states cause instantaneous jumps in energy.

This explains why we cannot use Gradient Descent. In such a landscape, gradient-based optimizers would stall immediately.

### 4. Why Does VAPO Work? (Convergence Mechanism)
Since there are no gradients, how does VAPO find the global minimum where $E=0$? The answer lies in the connectivity of algebraic space and the chaos of high-dimensional projections.

* **Algebraic Ergodicity:** The orbit structure of the Ideal Class Group $Cl(\Delta)$ is extremely rich. By applying perturbations $\epsilon$ of different norms, we can "teleport" from the current state $S$ to distant locations on the manifold.
* **Projection Sensitivity:** The `project_to_digit` function exhibits an "avalanche effect." A tiny change in the algebraic state $S$ (e.g., incrementing a coefficient $a$ by 1) leads to drastic changes in the generated action sequence.
* **Probabilistic Collision:** VAPO is essentially performing a guided stochastic collision. While we don't know which direction is "downhill," we know that by continuously jumping along group orbits (and retaining states where energy does not worsen), the Law of Large Numbers and orbital ergodicity ensure we eventually "crash" into the $E=0$ attractor basin.

### 5. Energy Aggregation Rules
The current energy aggregation logic is very simple (and harsh):
$$E_{total} = \sum E_{step}$$

As long as a single logical fallacy exists in the path, the total energy will be greater than 0. Evolver does not accept "partially correct" truths.

* **Target State:** $E_{total} = 0.0$
    (Absolute Truth).
* **Search Strategy:** As long as 
    $E > 0$,
    continue to Evolve.

### 6. Summary
Evolver’s definition of energy returns to the origin of logic: True vs. False. We do not attempt to "smooth out" logic (e.g., using $0.1$ to represent "slightly false"); instead, we embrace the discreteness of logic and leverage the complexity of algebraic groups to conquer it.
