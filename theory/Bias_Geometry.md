# THEORY PATCH: Geometry of the Bias Channel

## Formalizing the Action of $\vec{b}$ on the Manifold

### 1. The Geometric Ambiguity

The equation $z' = z + b$ is algebraically simple but geometrically ambiguous.

* If $z$ represents probability mass, this is invalid (breaks $\sum p = 1$).
* If $z$ represents unnormalized potential, what is the metric structure?

We resolve this by defining the **Bias Channel** using **Aitchison Geometry**.

---

### 2. The Logit Space as Tangent Bundle

Let $\Delta^{V-1}$ be the probability simplex of actions.
The **Logit Space** $\mathcal{L} \cong \mathbb{R}^V$ is interpreted as the **Tangent Space** $T_p \Delta^{V-1}$ (modulo the scaling invariance).

The **Softmax function** $\sigma: \mathbb{R}^V \to \Delta^{V-1}$ acts as the **Exponential Map** (projection from tangent space to manifold).

---

### 3. The Group Action: Affine Translation

The Bias Vector $\vec{b} \in \mathbb{R}^V$ acts on the Logit Space via the **Translation Group** $T(V)$:

$$Action_{\vec{b}}(z) = z + W_{proj} \cdot \vec{b}$$

#### 3.1 Equivalent Action on Simplex (Aitchison Perturbation)

This additive action on logits corresponds to a **Multiplicative Perturbation** on the simplex.
In Aitchison geometry, the operation $x \oplus y$ is defined as component-wise multiplication followed by closure:

$$p' = \sigma(z + b) \iff p' = \mathcal{C}(p \cdot e^b)$$

Where $\mathcal{C}$ is the closure (normalization) operator.

**Conclusion:** The Bias Controller is NOT performing linear interpolation on probabilities. It is performing **Log-Linear Modulation**.

---

### 4. Control Theory Implication

This definition is crucial for VAPO's convergence:

1.  **Unconstrained Optimization:** Since $\vec{b}$ acts on $\mathbb{R}^V$, VAPO does not need to respect the simplex boundary constraints ($\sum p = 1, p \ge 0$). The Softmax projection handles the geometry automatically.
2.  **Global Reachability:** Because the exponential map is surjective, an additive bias in $\mathbb{R}^V$ can reach any point in the interior of $\Delta^{V-1}$. This validates Theorem 5.7 (**Exact Controllability**).

---

### 5. Metric Consistency

Since VAPO calculates energy $E$ on the output (post-softmax logic) but optimizes $\vec{b}$ (pre-softmax logits), there is a metric distortion.

Let $J_\sigma$ be the Jacobian of Softmax. The gradient flow is:

$$\nabla_{\vec{b}} E = J_\sigma^T \cdot \nabla_{p} E$$

Because Softmax saturates, $J_\sigma \to 0$ for high-confidence predictions.

**Engineering Fix:** This explains why VAPO uses **Perturbation (Derivative-Free)** rather than Gradient Descent. When the generator is confident but wrong (Gradient Vanishing), a random jump in $\vec{b}$ (Logit space) can escape the saturation plateau, whereas gradients would die.
