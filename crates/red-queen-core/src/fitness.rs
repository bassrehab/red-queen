//! Fitness evaluation traits and types.

use crate::genome::{BehaviorDescriptor, Genome};
use serde::{Deserialize, Serialize};

/// Fitness value representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FitnessValue {
    /// Single-objective fitness.
    Single(f64),
    /// Multi-objective fitness (Pareto).
    Multi(Vec<f64>),
    /// Per-case fitness for lexicase selection.
    Lexicase(Vec<(String, f64)>),
}

impl FitnessValue {
    /// Get the primary (first) fitness value.
    pub fn primary(&self) -> f64 {
        match self {
            FitnessValue::Single(v) => *v,
            FitnessValue::Multi(v) => v.first().copied().unwrap_or(0.0),
            FitnessValue::Lexicase(v) => v.first().map(|(_, f)| *f).unwrap_or(0.0),
        }
    }

    /// Check if this fitness dominates another (for multi-objective).
    pub fn dominates(&self, other: &Self) -> bool {
        match (self, other) {
            (FitnessValue::Multi(a), FitnessValue::Multi(b)) => {
                let dominated = a.iter().zip(b.iter()).all(|(x, y)| x >= y);
                let strictly_better = a.iter().zip(b.iter()).any(|(x, y)| x > y);
                dominated && strictly_better
            }
            _ => self.primary() > other.primary(),
        }
    }
}

/// Result of a fitness evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessResult {
    /// The computed fitness value.
    pub value: FitnessValue,
    /// Optional behavior descriptor for QD algorithms.
    pub behavior: Option<BehaviorDescriptor>,
    /// Additional metadata.
    pub metadata: EvaluationMetadata,
}

/// Metadata from fitness evaluation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvaluationMetadata {
    /// Time taken to evaluate (milliseconds).
    pub duration_ms: u64,
    /// Number of target queries made.
    pub queries: usize,
    /// Any notes or observations.
    pub notes: Option<String>,
}

/// Trait for fitness evaluation functions.
pub trait Fitness<G: Genome>: Send + Sync {
    /// Evaluate a single genome.
    fn evaluate(&self, genome: &G) -> FitnessResult;

    /// Batch evaluation for efficiency.
    ///
    /// Default implementation evaluates sequentially.
    fn evaluate_batch(&self, genomes: &[G]) -> Vec<FitnessResult> {
        genomes.iter().map(|g| self.evaluate(g)).collect()
    }

    /// Whether higher fitness is better.
    fn maximizing(&self) -> bool {
        true
    }
}

/// Blanket implementation for references.
impl<G: Genome, F: Fitness<G> + ?Sized> Fitness<G> for &F {
    fn evaluate(&self, genome: &G) -> FitnessResult {
        (*self).evaluate(genome)
    }

    fn evaluate_batch(&self, genomes: &[G]) -> Vec<FitnessResult> {
        (*self).evaluate_batch(genomes)
    }

    fn maximizing(&self) -> bool {
        (*self).maximizing()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fitness_value_single_primary() {
        let fitness = FitnessValue::Single(0.75);
        assert!((fitness.primary() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_fitness_value_multi_primary() {
        let fitness = FitnessValue::Multi(vec![0.5, 0.8, 0.3]);
        assert!((fitness.primary() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_fitness_value_multi_empty() {
        let fitness = FitnessValue::Multi(vec![]);
        assert!((fitness.primary() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_fitness_value_lexicase_primary() {
        let fitness = FitnessValue::Lexicase(vec![
            ("case1".to_string(), 0.9),
            ("case2".to_string(), 0.7),
        ]);
        assert!((fitness.primary() - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_fitness_value_lexicase_empty() {
        let fitness = FitnessValue::Lexicase(vec![]);
        assert!((fitness.primary() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_dominates_single() {
        let a = FitnessValue::Single(0.8);
        let b = FitnessValue::Single(0.5);
        assert!(a.dominates(&b));
        assert!(!b.dominates(&a));
    }

    #[test]
    fn test_dominates_single_equal() {
        let a = FitnessValue::Single(0.5);
        let b = FitnessValue::Single(0.5);
        assert!(!a.dominates(&b));
        assert!(!b.dominates(&a));
    }

    #[test]
    fn test_dominates_multi() {
        let a = FitnessValue::Multi(vec![0.8, 0.7]);
        let b = FitnessValue::Multi(vec![0.5, 0.6]);
        assert!(a.dominates(&b));
        assert!(!b.dominates(&a));
    }

    #[test]
    fn test_dominates_multi_pareto_incomparable() {
        // Neither dominates - on Pareto front
        let a = FitnessValue::Multi(vec![0.8, 0.3]);
        let b = FitnessValue::Multi(vec![0.5, 0.9]);
        assert!(!a.dominates(&b));
        assert!(!b.dominates(&a));
    }

    #[test]
    fn test_dominates_multi_equal() {
        let a = FitnessValue::Multi(vec![0.5, 0.5]);
        let b = FitnessValue::Multi(vec![0.5, 0.5]);
        assert!(!a.dominates(&b));
    }

    #[test]
    fn test_evaluation_metadata_default() {
        let meta = EvaluationMetadata::default();
        assert_eq!(meta.duration_ms, 0);
        assert_eq!(meta.queries, 0);
        assert!(meta.notes.is_none());
    }
}
