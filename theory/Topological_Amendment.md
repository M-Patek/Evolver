# Amendment: The Topological Reconstruction 

**Problem ID:** DEADLOCK-HYPERBOLIC  
**Severity:** Critical  
**Status:** IMPLEMENTED  

---

## 1. The Diagnosis: Negative Curvature Deadlock

In previous versions, the VAPO optimizer treated the Cayley Graph as a Euclidean-like space. However, the Ideal Class Group (and especially the Quaternion Lattice) exhibits strong Hyperbolic Geometry in certain regions.

* **Symptoms:** Exponential divergence of paths. The number of possible "valid" semantic branches explodes.
* **The Trap:** Regions with Ricci Curvature $\kappa < 0$ (Tree-like divergence).
* **Result:** The optimizer gets stuck in high-entropy loops, unable to converge because every step increases the solution volume distance.

---

## 2. The Solution: Discrete Ricci Flow

We introduce a metric evolution operator $\mathcal{R}_t$ that dynamically reshapes the search space without altering the underlying algebraic topology.

### 2.1 Definition: Discrete Ollivier-Ricci Curvature

For any edge $e = (x, y)$, the coarse curvature is defined via the $W_1$ (Wasserstein) distance between neighborhoods:

$$\kappa(x, y) = 1 - \frac{W_1(m_x, m_y)}{d(x, y)}$$

* **$\kappa > 0$ (Spherical/Convergent):** Neighborhoods overlap significantly. Logic is robust.
* **$\kappa \approx 0$ (Euclidean/Flat):** Grid-like structure.
* **$\kappa < 0$ (Hyperbolic/Divergent):** Neighborhoods are disjoint. "Bridge" or "Tree" structure. High entropy risk.

### 2.2 The Flow Equation

We evolve the edge weights $w(e)$ over simulation time $t$:

$$\frac{d w_t(e)}{dt} = - \kappa(e) \cdot w_t(e)$$

* **Action:** If $\kappa < 0$ (Trap), then $\frac{dw}{dt} > 0$. The edge "lengthens".
* **Effect:** The "logical distance" to enter a high-entropy region becomes infinite. The optimizer perceives these regions as "far away" even if they are topologically adjacent.

---

## 3. Implementation Logic (VAPO Integration)

The VapoOptimizer now minimizes an Effective Energy $J_{eff}$:

$$J_{eff}(S) = J_{truth}(S) + \gamma \cdot \int_{Path} e^{-\kappa(s)} ds$$

In the code (`src/will/optimizer.rs` & `src/will/ricci.rs`):

1.  **Lookahead:** Before moving to state $S_{next}$, we calculate $\kappa(S_{curr}, S_{next})$.
2.  **Penalty:** We apply a penalty term `sensitivity * exp(-kappa)`.
3.  **Decision:** A move that slightly improves Semantic Truth ($E_{raw}$) but enters a chaotic region ($\kappa \ll 0$) will be rejected because $E_{eff}$ will increase.

This effectively implements **Conformal Flattening** on the discrete graph, forcing the Will to walk along the "Ridges of Stability".
