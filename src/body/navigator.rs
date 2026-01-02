use std::f64::consts::PI;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use crate::soul::algebra::ClassGroupElement;

/// NavigationFeatures represents the geometric invariants of an ideal class
/// projected onto a smooth cylindrical manifold.
/// 
/// Instead of using raw (a, b) coefficients which suffer from "reduction chaos",
/// we use trigonometric embeddings of the modular point tau = x + iy.
#[derive(Debug, Clone)]
pub struct NavigationFeatures {
    /// Cyclic embedding of the real part x = -b/2a
    /// Eliminates discontinuities from integer shifts (T-transformation)
    pub cos_x: f64,
    pub sin_x: f64,
    
    /// Logarithmic scaling of the imaginary part y = sqrt(|D|)/2a
    /// Converts multiplicative norm scales to additive distances
    pub log_y: f64,
}

impl NavigationFeatures {
    /// Extracts smooth geometric features from a rigorous algebraic state.
    pub fn extract(state: &ClassGroupElement) -> Self {
        // 1. Convert BigInt to f64 for geometric heuristic calculation.
        // Note: Precision loss here is acceptable as this is for the "Will" (Heuristic),
        // not the "Soul" (Verification). The rigorous state remains in BigInt.
        
        // Fallback to max value if overflow occurs (rare in search, possible in initialization)
        let a_f64 = state.a.to_f64().unwrap_or(f64::MAX); 
        let b_f64 = state.b.to_f64().unwrap_or(f64::MAX);
        let discrim_f64 = state.discriminant.to_f64().unwrap_or(f64::MIN); // Delta is negative
        
        // 2. Calculate the modular point tau = x + iy
        // x = -b / 2a
        let x = -b_f64 / (2.0 * a_f64);
        
        // y = sqrt(|Delta|) / 2a
        let y = discrim_f64.abs().sqrt() / (2.0 * a_f64);

        // 3. Project to Feature Space
        NavigationFeatures {
            cos_x: (2.0 * PI * x).cos(),
            sin_x: (2.0 * PI * x).sin(),
            log_y: y.ln(),
        }
    }

    /// Calculates the squared Euclidean distance in the feature manifold.
    /// This provides the "Gradient" for VAPO.
    pub fn distance_sq(&self, other: &Self) -> f64 {
        let d_cos = self.cos_x - other.cos_x;
        let d_sin = self.sin_x - other.sin_x;
        let d_log_y = self.log_y - other.log_y;
        
        d_cos * d_cos + d_sin * d_sin + d_log_y * d_log_y
    }
}
