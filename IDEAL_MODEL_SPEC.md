# IDEAL MODEL SPECIFICATION v1.0

"Detailed architectural blueprint for the Quaternion Architecture."

---

## 1. Overview

This document specifies the implementation details for **Evolver v1.0**.

## 2. Soul Layer: `src/soul/`

### 2.1 Algebraic Engine (`algebra.rs`)

* **Struct**: `Quaternion { a, b, c, d }` using `i64`.
* **Algebra**: Implements $B_{p, \infty}$ arithmetic with strict non-commutative multiplication.
* **Graph**: Implements `neighbors()` to generate edges of the **Pizer Graph** (Norm-$p$ elements).

### 2.2 Dynamics (`dynamics.rs`)

* **TimeEvolution**: Implements **Hecke Action** instead of simple squaring.
* **Causality**: The `next()` function applies a generator from the right: $q_{t+1} = q_t \cdot g$.

### 2.3 Spectral Governor (`governor.rs`)

* **Role**: Runtime topology analysis.
* **Algorithm**: Power Iteration (with deflation) to estimate $\lambda_2$ of the visited subgraph.
* **Trigger**: If the graph becomes "linear" (gap $\to 0$), initiates a `migrate_algebra()` call.

### 2.4 State Lifter (`lifter.rs`)

* **Role**: Handling $p$-change events.
* **Process**:
    1.  **Project**: Map current Quaternion to a feature vector (conceptually Modular Forms).
    2.  **Transport**: Instantiate new `QuaternionAlgebra` with new $p$.
    3.  **Fine-Tune**: Use **Beam Search** (Approximate CVP) to find the closest lattice point in the new algebra.

---

## 3. Will Layer: `src/will/`

### 3.1 Perturber (`perturber.rs`)

* **Implementation**: `HeckePerturber`.
* **Output**: A vector of Quaternions $\{g_1, g_1^{-1}, g_2, g_2^{-1}, \dots\}$ representing the valid moves in the Pizer graph.

---

## 4. Body Layer: `src/body/`

### 4.1 Topology Interface

* **Feature Projector**: Maps algebraic states to high-dimensional feature vectors to calculate $E_{obj}$.
* **Lipschitz Map**: Adapted to measure distances in the Quaternion metric $N(q_1 - q_2)$.

---

## Summary

The v1.0 implementation relies on **"Dynamic Exploration on a Ramanujan Graph"**. The addition of the Governor and Lifter makes the system robust against local minima by allowing it to change the fundamental constants of its universe.
