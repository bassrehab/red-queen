//! Variation operators (mutation and crossover).

use crate::genome::Genome;
use rand::Rng;

/// Configuration for variation.
#[derive(Debug, Clone)]
pub struct VariationConfig {
    /// Probability of mutation.
    pub mutation_rate: f64,
    /// Probability of crossover.
    pub crossover_rate: f64,
}

impl Default for VariationConfig {
    fn default() -> Self {
        Self {
            mutation_rate: 0.3,
            crossover_rate: 0.7,
        }
    }
}

/// Apply variation to produce offspring.
pub fn vary<G: Genome, R: Rng>(
    parent1: &G,
    parent2: &G,
    config: &VariationConfig,
    rng: &mut R,
) -> G {
    // Crossover
    let mut offspring = if rng.gen::<f64>() < config.crossover_rate {
        parent1.crossover(parent2, rng)
    } else {
        parent1.clone()
    };

    // Mutation
    offspring.mutate(rng, config.mutation_rate);

    offspring
}
