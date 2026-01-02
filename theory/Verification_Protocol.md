# Verification Protocols: Proof of Will & Evolution

## 1. Overview
To fulfill the promise of Evolver’s "Formally Verifiable Search," we define a set of cryptography-friendly verification protocols. This protocol aims to solve two fundamental problems:
* **Proof of Will (Search Proof):** How to prove that an Agent actually found a low-energy state through computation, rather than fabricating data out of thin air?
* **Integrity of Evolution:** How to ensure that the evolution process strictly follows predefined physical laws (STP dynamics)?

## 2. Notation
* $\mathcal{S}$: State Space.
* $\mathcal{U}$: Control/Perturbation Space.
* $M$: The Structure Matrix of the system, encoding all physical laws.
* $f(x, u) = M \ltimes u \ltimes x$: Deterministic STP (Semi-Tensor Product) dynamic equations.
* $E(x)$: Energy/Evaluation function.
* $H(\cdot)$: Collision-resistant Hash Function (e.g., SHA-256).

## 3. Protocol A: Proof of Will
**Objective:** To verify that the optimizer's (Prover's) claim—"I have found a path to a low-entropy state"—is authentic.

### 3.1 The Certificate (Data Structure)
The proof $\pi_{will}$ is defined as a tuple:
$$\pi_{will} = (x_0, \mathcal{U}_{seq}, E_{claim}, h_{dyn})$$
Where:
* $x_0$: Initial state.
* $\mathcal{U}_{seq} = [u_0, u_1, \dots, u_k]$: A sequence of actions/perturbations.
* $E_{claim}$: The claimed final energy.
* $h_{dyn} = H(M)$: The Dynamics Fingerprint (hash of the Structure Matrix).

### 3.2 Verifier Algorithm
The Verifier $V$ takes $(\pi_{will}, M, E(\cdot))$ as input and performs the following steps:
1.  **Fingerprint Check:**
    $$\text{Assert } H(M) == \pi_{will}.h_{dyn}$$
    This ensures the physical laws used by the Verifier are identical to those used by the Optimizer when generating the path.
2.  **Deterministic Replay:**
    * Set $x_{curr} = x_0$.
    * For $i = 0$ to $k$:
        $$x_{next} = M \ltimes u_i \ltimes x_{curr}$$
        Update $x_{curr} \leftarrow x_{next}$.
    * **Note:** STP multiplication is strictly deterministic with zero randomness.
3.  **Energy Audit:**
    $$\text{Assert } |E(x_{curr}) - \pi_{will}.E_{claim}| < \epsilon$$

### 3.3 Complexity Analysis
* **Prover (Optimization):** $O(\text{NP-Hard})$ or $O(\text{Heuristic})$. Finding the sequence $u$ usually requires searching in a massive topological space, which is computationally expensive.
* **Verifier (Replay):** $O(k \cdot C_{stp})$, where $k$ is the number of steps and $C_{stp}$ is the cost of a single STP matrix multiplication.
* **Asymmetry:** Verification cost is significantly lower than generation cost. This makes the protocol suitable for distributed verification scenarios.

## 4. Protocol B: Proof of Evolution (Time/Evolution Proof)
**Objective:** To prove that the state $x_{final}$ was derived from $x_0$ through an actual passage of time (dynamic iterations), preventing "teleportation" cheating.

### 4.1 The Core Challenge
In the STP architecture, if there are no external perturbations ($u_t = I$), the dynamics simplify to $x_{t+1} = M \ltimes x_t$. After $T$ steps, $x_T = M^T \ltimes x_0$.
If the Verifier has massive computing power, they could compute $M^T$ using Repeated Squaring in $O(\log T)$ time, thus losing the significance of the "passage of time" proof.

### 4.2 Sequentiality Enforcement
To make the Proof of Evolution function like a VDF (Verifiable Delay Function), we require that the dynamic evolution must include unpredictable perturbations $u_t$, or that the structure of $M$ is sufficiently complex to make pre-computation of $M^T$ unfeasible.

In the current version of Evolver, we rely on **Path Dependency**:
Since each perturbation $u_t$ is dynamically determined by the optimizer based on the current state $x_t$ ($u_t = \text{Will}(x_t)$), the Verifier cannot pre-calculate $M^T$.

Therefore, verification is inherently serialized:
$$x_1 = M \ltimes u_0 \ltimes x_0 \rightarrow x_2 = M \ltimes u_1 \ltimes x_1 \dots$$
**Conclusion:** As long as the perturbation sequence $\mathcal{U}_{seq}$ is non-trivial, the verification process is Inherently Sequential and cannot be accelerated through parallelization.

## 5. Security Assumptions
1.  **Soundness of STP:** Assumes the implementation of the STP algebraic system is bug-free, meaning $M \ltimes x$ always produces a valid physical state.
2.  **Determinism:** Given the same $M, x, u$, the results must be bit-wise consistent across any hardware platform. Note: Floating-point calculations require special handling or the use of fixed-point/integer logic in core verification to ensure cross-platform consistency.
3.  **Hash Collision Resistance:** An attacker cannot construct another $M'$ such that $H(M') = H(M)$ while $E(M', \dots) = 0$.

## 6. Summary
Evolver’s verification protocol is built upon **Computational Asymmetry**.
* **Will** is responsible for consuming computational power to find a path within a complex energy landscape.
* **Verifier (Observer)** is responsible for retracing the path at a linear cost to confirm its physical legitimacy.

This separation ensures that even if the Optimizer (Will) is based on black-box neural networks or random guesses, the system's final output (Body's State) remains rigorous and trustworthy.
