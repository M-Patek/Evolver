# Constraint Semantics: The Rules of Algebra

## 1. The Nature of Constraints
In this architecture, a "Constraint" is no longer an additional rule imposed on an external generator, but rather an intrinsic property of the algebraic structure itself.
Constraints define validity. Only state sequences that satisfy specific algebraic and logical rules can be recognized as "Truth."

## 2. Formal Definition

### 2.1 Algebraic Constraints
These constraints are enforced by the mathematical core (`src/soul/algebra.rs`).

**Discriminant Invariance:**
For any state 
$S_t = (a_t, b_t, c_t)$ 
on the evolution trajectory, the following must hold:
$$b_t^2 - 4a_t c_t = \Delta$$

**Group Closure:**
The result of any operation 
$S_{new} = S_{old} \circ \epsilon$ 
must remain within the Ideal Class Group 
$Cl(\Delta)$. 
This guarantees that the system never produces "mathematically nonsensical" states.

### 2.2 Logical Constraints
These constraints are verified by the STP engine (`src/dsl/stp_bridge.rs`).

**Type Consistency:**
The action `Define { symbol: "n", type: "Odd" }` must comply with the rules of the type system.

**Causal Consistency:**
The action `Apply { inputs: ["n"] }` requires that the symbol "n" must have been defined in a previous step.
$$a_t \text{ is valid} \iff \text{Preconditions}(a_t) \subseteq \bigcup_{i=0}^{t-1} \text{Effects}(a_i)$$

**Axiomatic Consistency:**
The assertion `Assert { condition }` must evaluate to true under the current STP state.
$$E_{STP}(S_t, a_t) = 0 \iff \text{STP}(S_t) \vdash a_t$$

## 3. Constraint Manifold
We define the set of all states that satisfy the constraints as the **Manifold of Truth** $\mathcal{M}_{truth}$.
$$\mathcal{M}_{truth} = \{ \tau \in (Cl(\Delta))^* \mid \forall t, E_{STP}(\Psi(S_t)) = 0 \}$$

The task of the optimizer (VAPO) is to restrict the system's trajectory to this manifold.

* **Hard Constraints:** Must be satisfied absolutely (e.g., the Discriminant). If violated, the code will Panic or return an error.
* **Soft Constraints:** Can be temporarily violated during the search process (e.g., STP energy), but the final solution must satisfy them.

## 4. Code Mapping
* `ClassGroupElement::compose`: Guarantees algebraic constraints.
* `STPContext::calculate_energy`: Checks logical constraints. If violated, it returns non-zero energy, guiding VAPO to avoid that path.
