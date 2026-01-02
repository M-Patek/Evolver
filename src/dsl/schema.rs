// src/dsl/schema.rs

/// The schema definition for the Evolver Domain Specific Language (DSL).
/// Defines the core structures for Logic Matrices and Constraints.

use serde::{Deserialize, Serialize};

/// Represents the algebraic logic matrix operations.
/// In Evolver, logic is treated as continuous energy functions.
pub struct LogicMatrix;

impl LogicMatrix {
    /// Calculates the energy penalty for logical implication A -> B (A implies B).
    ///
    /// Logic: A -> B is equivalent to !A or B.
    /// In Soft Logic Energy terms:
    /// - If A is True (1.0) and B is False (0.0), this is a VIOLATION. Energy should be HIGH.
    /// - If A is False (0.0), the statement is "vacuously true". Energy should be LOW.
    /// - If A is True and B is True, the statement is true. Energy should be LOW.
    ///
    /// Formula: Penalty = P(A) * (1 - P(B)) * ScalingFactor
    pub fn implies_energy(prob_a: f64, prob_b: f64) -> f64 {
        // Scaling factor of 10.0 provides a steeper gradient for the VAPO optimizer
        // to detect violations more easily.
        prob_a * (1.0 - prob_b) * 10.0
    }
}

/// Basic predicates that can be asserted on a variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Predicate {
    IsOdd,
    IsEven,
    IsPrime,
    IsPositive,
    // Future expansion: relational predicates like GreaterThan(f64)
}

/// Strategies for aggregating energy across a collection (for Quantifiers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Strict Summation: Any violation adds to the total energy.
    /// Good for ensuring *absolute* compliance of all elements.
    Sum,
    
    /// LogSumExp (Softmax): The energy is dominated by the worst offender.
    /// Good for gradient-based guidance, directing focus to the biggest error first.
    LogSumExp,
}

/// The core Constraint enum defining all possible logical rules in the DSL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    /// Basic assertion: "Variable X must satisfy Predicate P"
    /// Example: Assert(IsOdd, "x")
    Assert(Predicate, String),
    
    /// Logical Implication: "If A is true, then B must be true"
    /// Example: AssertImplies("is_raining", "is_cloudy")
    AssertImplies(String, String),

    /// Universal Quantifier (The Aggregator): "For all x in Collection, P(x) is true"
    /// This "unrolls" the loop into a single energy value.
    /// Example: AssertForAll { collection: "SmallPrimes", predicate: IsOdd, strategy: Sum }
    AssertForAll {
        collection: String, // ID of the collection in the context
        predicate: Predicate,
        strategy: AggregationStrategy,
    },
}
