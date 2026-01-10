//! Selection operators.

use crate::genome::Genome;
use crate::population::{Individual, Population};
use rand::Rng;

/// Trait for selection operators.
pub trait Selection<G: Genome>: Send + Sync {
    /// Select one parent from the population.
    fn select<'a, R: Rng>(
        &self,
        population: &'a Population<G>,
        rng: &mut R,
    ) -> &'a Individual<G>;
}

/// Tournament selection.
pub struct Tournament {
    /// Tournament size.
    pub size: usize,
}

impl Tournament {
    /// Create a new tournament selector.
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl<G: Genome> Selection<G> for Tournament {
    fn select<'a, R: Rng>(
        &self,
        population: &'a Population<G>,
        rng: &mut R,
    ) -> &'a Individual<G> {
        let n = population.individuals.len();
        let mut best: Option<&Individual<G>> = None;
        let mut best_fitness = f64::NEG_INFINITY;

        for _ in 0..self.size {
            let idx = rng.gen_range(0..n);
            let ind = &population.individuals[idx];
            let fitness = ind.fitness_value();

            if fitness > best_fitness {
                best_fitness = fitness;
                best = Some(ind);
            }
        }

        best.unwrap_or(&population.individuals[0])
    }
}

// TODO: Implement Lexicase, Novelty, NSGA2 selection
