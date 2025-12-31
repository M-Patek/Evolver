# Optimization Problem: Discrete Search on Cayley Graphs

## 1. The Paradigm Shift: From Manifolds to Graphs

We formally abandon the "continuous manifold" analogy. The Class Group $Cl(\Delta)$ is a finite Abelian group. The optimization problem is strictly defined as a Search Problem on a Cayley Graph.

There are no gradients. There is only connectivity.

## 2. Problem Formulation

### 2.1 The State Space: Cayley Graph $\mathcal{G}$

Vertices $V$: All reduced binary quadratic forms of discriminant $\Delta$.

Generators $\mathcal{P}$: A subset of prime forms defined by the VAPO perturber.

$$
\mathcal{P} = \{ \mathfrak{p}_2, \mathfrak{p}_3, \mathfrak{p}_5, \dots, \mathfrak{p}_{max} \}
$$

(where $\mathfrak{p}_q$ denotes a prime ideal form of norm $q$).

Edges $E$: Two states $S_1, S_2$ are connected if $S_2 = S_1 \circ \epsilon$ for some $\epsilon \in \mathcal{P}$.

### 2.2 Objective Function

The objective is defined on the vertices of the graph:

$$
\text{Minimize } J(S) = E_{STP}(\Psi(S))
$$

Where $J: V(\mathcal{G}) \to \{0, 1, \dots \}$ is the discrete energy potential.

## 3. Search Dynamics & Discrete Curvature

### 3.1 Neighborhood $\mathcal{N}(S)$

The 1-hop neighborhood of a state $S$ is explicitly defined by the generator set:

$$
\mathcal{N}(S) = \{ S \circ \epsilon \mid \epsilon \in \mathcal{P} \}
$$

### 3.2 The "Gradient" Proxy

Since $\nabla J$ does not exist, we define a Discrete Descent Vector based on energy differences in the neighborhood:

$$
\Delta J(\epsilon) = J(S \circ \epsilon) - J(S)
$$

VAPO selects the edge $\epsilon^*$ that minimizes this difference (Steepest Descent on Graph).

### 3.3 Theoretical Extension: Discrete Curvature

(Future Implementation Note)
While currently VAPO uses greedy descent, the graph structure allows for the definition of Ollivier-Ricci Curvature.

* **Positive Curvature:** Neighborhoods contract (easier to converge, "convex-like").
* **Negative Curvature:** Neighborhoods expand (tree-like, harder to converge).

In regions of negative curvature (which are common in Class Group graphs due to their chaotic mixing properties), greedy search may fail. VAPO mitigates this by allowing "Thermal Fluctuations" (Metropolis acceptance of bad states) to traverse negatively curved bridges.

## 4. Solver Algorithm: VAPO (Graph Walker Variant)

The algorithm is now formally a Local Search with Variable Neighborhood Descent.

**Algorithm Flow:**

1.  **Initialization:** $S_0$ is the Seed Node.
2.  **Iteration:**
    * **Dynamic Generator Selection:** Instead of "Valuation", we define generator subsets $\mathcal{P}_{coarse} \subset \mathcal{P}$ and $\mathcal{P}_{fine} \subset \mathcal{P}$.
        * $\mathcal{P}_{coarse}$: Generators that empirically map to high-impact digits in $\Psi$.
        * $\mathcal{P}_{fine}$: Generators that map to low-impact digits.
    * **Neighbor Generation:** Construct $\mathcal{N}_{active}(S_t)$ using the current subset.
    * **Transition:** Move to $S_{t+1} \in \mathcal{N}_{active}$ that minimizes $J$.

## 5. Why this works?

Class Groups are Expanders (conjectured). Random walks on expander graphs mix rapidly. VAPO is a "Guided Walk" on an expander graph, which explains why it can find rare solutions ($E=0$) relatively quickly despite the enormous size of the group.
