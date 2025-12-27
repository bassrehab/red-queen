//! Genome trait and utilities.
//!
//! A genome is the evolvable representation of a solution.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Core trait for evolvable representations.
///
/// Implement this trait to define how your solutions are represented,
/// mutated, and combined.
pub trait Genome: Clone + Send + Sync + Sized {
    /// The phenotype type - what the genome produces when expressed.
    type Phenotype;

    /// Create a random genome.
    fn random<R: Rng>(rng: &mut R) -> Self;

    /// Mutate the genome in place.
    ///
    /// # Arguments
    /// * `rng` - Random number generator
    /// * `rate` - Mutation rate (0.0 to 1.0)
    fn mutate<R: Rng>(&mut self, rng: &mut R, rate: f64);

    /// Combine two genomes to produce offspring.
    ///
    /// # Arguments
    /// * `other` - The other parent genome
    /// * `rng` - Random number generator
    fn crossover<R: Rng>(&self, other: &Self, rng: &mut R) -> Self;

    /// Convert genome to its phenotype (the actual solution/input).
    fn to_phenotype(&self) -> Self::Phenotype;

    /// Compute distance to another genome (for novelty search).
    ///
    /// Default implementation returns 0.0 (no distance defined).
    fn distance(&self, _other: &Self) -> f64 {
        0.0
    }
}

/// Behavior descriptor for quality-diversity algorithms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehaviorDescriptor {
    /// Values for each behavior dimension.
    pub values: Vec<f64>,
}

impl BehaviorDescriptor {
    /// Create a new behavior descriptor.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    /// Compute Euclidean distance to another descriptor.
    pub fn distance(&self, other: &Self) -> f64 {
        self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_distance() {
        let a = BehaviorDescriptor::new(vec![0.0, 0.0]);
        let b = BehaviorDescriptor::new(vec![3.0, 4.0]);
        assert!((a.distance(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_behavior_distance_same_point() {
        let a = BehaviorDescriptor::new(vec![1.0, 2.0, 3.0]);
        let b = BehaviorDescriptor::new(vec![1.0, 2.0, 3.0]);
        assert!((a.distance(&b)).abs() < 1e-10);
    }

    #[test]
    fn test_behavior_distance_single_dimension() {
        let a = BehaviorDescriptor::new(vec![0.0]);
        let b = BehaviorDescriptor::new(vec![5.0]);
        assert!((a.distance(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_behavior_descriptor_new() {
        let desc = BehaviorDescriptor::new(vec![1.0, 2.0, 3.0]);
        assert_eq!(desc.values, vec![1.0, 2.0, 3.0]);
    }
}
