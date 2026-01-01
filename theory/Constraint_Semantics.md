# Constraint Semantics: The Rules of Algebra

## 1. The Nature of Constraints

In this architecture, a "Constraint" is no longer an additional rule imposed on an external generator, but rather an intrinsic property of the algebraic structure itself.
Constraints define validity. Only state sequences that satisfy specific algebraic and logical rules can be recognized as "Truth."

## 2. The Dual Binding Principle

To ensure that the generation process represents genuine reasoning, we enforce a Dual Binding between the Semantics (Context) and the Mathematics (State).

### 2.1 The Physical Binding (Context $\to$ $\Delta$)

The prompt is not just an input; it is the Genesis of the mathematical universe.

**Mechanism:**
$$\Delta = \text{HashToPrime}(Context)$$

**Effect:** The specific "terrain" of the Cayley Graph (its diameter, cycles, and connectivity) is physically shaped by the meaning of the question.

**Result:** You cannot traverse the map of "Question A" to find the answer to "Question B". They exist in disjoint mathematical universes.

### 2.2 The Teleological Binding (Context $\to$ $E_{STP}$)

The prompt defines the "Gravity" (Energy Potential) of the universe.

**Mechanism:** The Zero-Energy Manifold $\mathcal{M}_{truth}$ is defined strictly by the logical assertions required by the Context.

**Effect:** A path is only "downhill" (energetically favorable) if it semantically aligns with the prompt's requirements.

**Result:** The "Will" (Optimizer) is forced to reason (search) towards the specific semantic goal, because that is the only way to lower its energy state.

---

## 3. Formal Definition

### 3.1 Algebraic Constraints (Hard)

These constraints are enforced by the mathematical core (`src/soul/algebra.rs`).

**Discriminant Invariance:**
For any state
$$S_t = (a_t, b_t, c_t)$$
on the evolution trajectory, the following must hold:
$$b_t^2 - 4a_t c_t = \Delta$$

**Group Closure:**
The result of any operation
$$S_{new} = S_{old} \circ \epsilon$$
must remain within the Ideal Class Group
$$Cl(\Delta)$$

### 3.2 Logical Constraints (Soft)

These constraints are verified by the STP engine (`src/dsl/stp_bridge.rs`).

**Type Consistency:**
The action `Define { symbol: "n", type: "Odd" }` must comply with the rules of the type system.

**Causal Consistency:**
The action `Apply { inputs: ["n"] }` requires that the symbol "n" must have been defined in a previous step.
$$a_t \text{ is valid} \iff \text{Preconditions}(a_t) \subseteq \bigcup_{i=0}^{t-1} \text{Effects}(a_i)$$

**Axiomatic Consistency:**
The assertion `Assert { condition }` must evaluate to true under the current STP state.
$$E_{STP}(S_t, a_t) = 0 \iff \text{STP}(S_t) \vdash a_t$$

---

## 4. Constraint Manifold & Search Dynamics

To resolve the duality between algebraic rigour and heuristic search, we explicitly distinguish between the Search Space (where the Will travels) and the Manifold of Truth (where the Will aims to arrive).

### 4.1 The Manifold of Truth (Destination)

The Manifold of Truth
$$\mathcal{M}_{truth}$$
is the sparse subset of the class group containing seeds that materialize into logically valid paths for the specific context.
$$\mathcal{M}_{truth}(\Delta) = \{ S \in Cl(\Delta) \mid E_{STP}(\text{Materialize}(S) \mid \text{Context}) = 0 \}$$

This is the target set. A seed
$$S^*$$
is considered a valid solution if and only if
$$S^* \in \mathcal{M}_{truth}(\Delta)$$

### 4.2 The Search Space (Journey)

The Search Space is the entire Cayley Graph
$$\mathcal{G}(\Delta)$$

During the optimization process (The Will's Walk), the system traverses states
$$S_{temp}$$
that generally do not belong to $\mathcal{M}_{truth}$.
$$S_{temp} \in \mathcal{G} \setminus \mathcal{M}_{truth} \implies E_{STP}(S_{temp}) > 0$$

The task of the optimizer (VAPO) is to navigate the graph to converge onto the manifold, minimizing the energy potential
$$J(S)$$

---

## 5. Code Mapping

* `ClassGroupElement::compose`: Guarantees algebraic constraints (Hard).
* `STPContext::calculate_energy`: Checks logical constraints (Soft). If violated, it returns non-zero energy, guiding VAPO to avoid that path.
* `(Implicit)`: The `Evolver` struct initialization binds the Discriminant to the input prompt, enforcing the Contextual Binding at the root level.
