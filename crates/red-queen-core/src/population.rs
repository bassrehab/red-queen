//! Population management.

use crate::fitness::FitnessValue;
use crate::genome::{BehaviorDescriptor, Genome};
use serde::{Deserialize, Serialize};

/// An individual in the population.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Individual<G: Genome> {
    /// The genome.
    pub genome: G,
    /// Fitness value (None if not yet evaluated).
    pub fitness: Option<FitnessValue>,
    /// Behavior descriptor for QD algorithms.
    pub behavior: Option<BehaviorDescriptor>,
    /// Generation this individual was created.
    pub birth_generation: usize,
}

impl<G: Genome> Individual<G> {
    /// Create a new individual with a genome.
    pub fn new(genome: G) -> Self {
        Self {
            genome,
            fitness: None,
            behavior: None,
            birth_generation: 0,
        }
    }

    /// Get the primary fitness value.
    pub fn fitness_value(&self) -> f64 {
        self.fitness.as_ref().map(|f| f.primary()).unwrap_or(0.0)
    }
}

/// Configuration for population.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    /// Population size.
    pub size: usize,
    /// Number of elite individuals to preserve.
    pub elitism: usize,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            size: 100,
            elitism: 5,
        }
    }
}

/// A population of individuals.
#[derive(Debug, Clone)]
pub struct Population<G: Genome> {
    /// The individuals.
    pub individuals: Vec<Individual<G>>,
    /// Current generation.
    pub generation: usize,
    /// Configuration.
    pub config: PopulationConfig,
}

impl<G: Genome> Population<G> {
    /// Create a new random population.
    pub fn random<R: rand::Rng>(config: PopulationConfig, rng: &mut R) -> Self {
        let individuals = (0..config.size)
            .map(|_| Individual::new(G::random(rng)))
            .collect();

        Self {
            individuals,
            generation: 0,
            config,
        }
    }

    /// Get the best individual by fitness.
    pub fn best(&self) -> Option<&Individual<G>> {
        self.individuals
            .iter()
            .filter(|i| i.fitness.is_some())
            .max_by(|a, b| {
                a.fitness_value()
                    .partial_cmp(&b.fitness_value())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get the top N individuals by fitness.
    pub fn top_n(&self, n: usize) -> Vec<&Individual<G>> {
        let mut sorted: Vec<_> = self.individuals.iter().collect();
        sorted.sort_by(|a, b| {
            b.fitness_value()
                .partial_cmp(&a.fitness_value())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(n).collect()
    }
}
