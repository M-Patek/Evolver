# THEORY PATCH: The Neuro-Symbolic Fibration

Closing the Structural Hole between $\mathbb{R}^V$, $A$, and $S$ via Verbalization

## 1. Problem Identification: The Type Gap

The previous iteration of this theory committed a category error by conflating the Token Manifold ($\mathbb{R}^V$) with the Logical Fiber ($A|_s$).

* **The Manifold $L \cong \mathbb{R}^V$**: The continuous space of raw logits (Log-probabilities) over the Vocabulary. Dimension $V \approx 32,000+$.
* **The Lattice $A$**: The discrete set of valid ProofActions (Schema Objects).
* **The Module $S$**: The STP algebraic state space.

**The Fix**: We introduce an explicit Verbalizer Map ($\nu$) to bridge the continuous manifold of the LLM and the discrete fiber of the Algebra.

---

## 2. The Mathematical Closure: Fiber Bundles with Verbalization

We redefine the system as a Verbalized Principal Bundle.

### 2.1 The Base Space: Algebraic Truth ($S$)

Let the STP State Space $S$ be the Base Manifold.
Points $s \in S$ are valid algebraic states.
Movement is governed by:

$$s_{t+1} = M \ltimes u \ltimes s_t$$

### 2.2 The Discrete Fiber: Action Schemas ($A|_s$)

At any state $s \in S$, we define the Admissible Action Fiber:

$$A|_s = \{ a \in A_{univ} \mid \text{Energy}(s, a) < \epsilon \}$$

This creates a discrete Sheaf of Actions over $S$.

### 2.3 The Missing Link: The Verbalizer ($\nu$)

This was the missing component. We cannot directly compare a Logit Vector $z \in \mathbb{R}^V$ to an Action Object $a \in A|_s$.

We define the Verbalization Map (or Action Embedding) $\nu$:

$$\nu: A_{univ} \to \mathbb{R}^V$$

For a given action $a$ (e.g., Define { symbol: "n", ... }), $\nu(a)$ returns its Prototype Vector in the vocabulary space.

* **In Engineering**: This is the embedding of the token sequence corresponding to the action, or a mask vector.
* **In Topology**: This maps the discrete fiber nodes into points (or regions) in the continuous manifold $\mathbb{R}^V$.

---

## 3. The Unifying Morphism: The "Section"

In this formalism, the "Generator" (LLM) is a Section ($\sigma$) that points towards the verbalized fiber.

$$\sigma: S \to \mathbb{R}^V$$

### 3.1 The Projection (Decoder): Pullback via Verbalization

The Decoder $\Pi$ is no longer a naive Voronoi partition of $\mathbb{R}^V$. It is defined as the Argmax of the Pullback:

Given a generated logit vector $z = \sigma(s)$, the decoder selects the action $a^*$ that maximizes alignment with the verbalizer:

$$\Pi(z, s) = \underset{a \in A|_s}{\text{argmax}} \langle z, \nu(a) \rangle$$

**Constraint**: The search is strictly limited to $a \in A|_s$ (the valid fiber).

**Geometry**: This defines a dynamic Voronoi tessellation on $\mathbb{R}^V$ where the "centers" are the verbalized points $\{ \nu(a) \}_{a \in A|_s}$.

### 3.2 The Transition (STP Dynamics)

The loop is closed via:

$$s_{t+1} = F(\Pi(\sigma(s), s)) \cdot s_t$$

---

## 4. Closing the Loop: The Bias Connection

How does VAPO control this?

The Bias Vector $\vec{b}$ is an optimization variable acting on the Verbalized Manifold.

* **Source**: VAPO calculates $\vec{b}_{ctrl}$ in the abstract control space.
* **Verbalization**: The system projects this bias into the token space: 

$$\vec{b}_{logits} = W_{proj} \cdot \vec{b}_{ctrl}$$

* **Note**: $W_{proj}$ serves as the linear approximation of the Verbalizer $\nu$ for the bias signal.
* **Action**: The generator output is perturbed: 

$$z' = z + \vec{b}_{logits}$$

* **Steering**: The perturbation shifts $z$ across the decision boundary of the Voronoi cells defined by $\Pi(\cdot, s)$.

**Closure Theorem (Revised)**:
Control is possible if the image of the Bias Verbalizer ($Im(W_{proj})$) has non-zero intersection with the normal vectors of the Voronoi boundaries in $\mathbb{R}^V$.

$$\exists \vec{b}, \quad \Pi(\sigma(s) + W_{proj}\vec{b}, s) = a_{target}$$

This confirms that we control the Logical Outcome by mechanically pushing the Token Distribution towards the Verbalized Prototype of the desired action.
