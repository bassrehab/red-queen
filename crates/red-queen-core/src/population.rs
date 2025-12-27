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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    // Simple test genome
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct TestGenome {
        value: f64,
    }

    impl Genome for TestGenome {
        type Phenotype = f64;

        fn random<R: rand::Rng>(rng: &mut R) -> Self {
            Self {
                value: rng.gen_range(0.0..1.0),
            }
        }

        fn mutate<R: rand::Rng>(&mut self, rng: &mut R, _rate: f64) {
            self.value = rng.gen_range(0.0..1.0);
        }

        fn crossover<R: rand::Rng>(&self, other: &Self, _rng: &mut R) -> Self {
            Self {
                value: (self.value + other.value) / 2.0,
            }
        }

        fn to_phenotype(&self) -> f64 {
            self.value
        }
    }

    #[test]
    fn test_individual_new() {
        let genome = TestGenome { value: 0.5 };
        let ind = Individual::new(genome);
        assert!(ind.fitness.is_none());
        assert!(ind.behavior.is_none());
        assert_eq!(ind.birth_generation, 0);
    }

    #[test]
    fn test_individual_fitness_value_none() {
        let genome = TestGenome { value: 0.5 };
        let ind = Individual::new(genome);
        assert!((ind.fitness_value() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_individual_fitness_value_some() {
        let genome = TestGenome { value: 0.5 };
        let mut ind = Individual::new(genome);
        ind.fitness = Some(FitnessValue::Single(0.75));
        assert!((ind.fitness_value() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_population_config_default() {
        let config = PopulationConfig::default();
        assert_eq!(config.size, 100);
        assert_eq!(config.elitism, 5);
    }

    #[test]
    fn test_population_random_size() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 20,
            elitism: 2,
        };
        let pop: Population<TestGenome> = Population::random(config, &mut rng);
        assert_eq!(pop.individuals.len(), 20);
        assert_eq!(pop.generation, 0);
    }

    #[test]
    fn test_population_best_none_evaluated() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 10,
            elitism: 1,
        };
        let pop: Population<TestGenome> = Population::random(config, &mut rng);
        assert!(pop.best().is_none());
    }

    #[test]
    fn test_population_best_with_fitness() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 10,
            elitism: 1,
        };
        let mut pop: Population<TestGenome> = Population::random(config, &mut rng);

        // Assign fitness values
        for (i, ind) in pop.individuals.iter_mut().enumerate() {
            ind.fitness = Some(FitnessValue::Single(i as f64));
        }

        let best = pop.best().unwrap();
        assert!((best.fitness_value() - 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_population_top_n() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 10,
            elitism: 1,
        };
        let mut pop: Population<TestGenome> = Population::random(config, &mut rng);

        for (i, ind) in pop.individuals.iter_mut().enumerate() {
            ind.fitness = Some(FitnessValue::Single(i as f64));
        }

        let top3 = pop.top_n(3);
        assert_eq!(top3.len(), 3);
        assert!((top3[0].fitness_value() - 9.0).abs() < 1e-10);
        assert!((top3[1].fitness_value() - 8.0).abs() < 1e-10);
        assert!((top3[2].fitness_value() - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_population_top_n_larger_than_population() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 5,
            elitism: 1,
        };
        let pop: Population<TestGenome> = Population::random(config, &mut rng);

        let top10 = pop.top_n(10);
        assert_eq!(top10.len(), 5); // Only 5 individuals exist
    }
}
