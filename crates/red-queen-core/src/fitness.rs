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
