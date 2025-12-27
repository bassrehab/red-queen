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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fitness::FitnessValue;
    use crate::population::PopulationConfig;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    // Simple test genome
    #[derive(Clone)]
    struct TestGenome {
        value: f64,
    }

    impl Genome for TestGenome {
        type Phenotype = f64;

        fn random<R: Rng>(rng: &mut R) -> Self {
            Self {
                value: rng.gen_range(0.0..1.0),
            }
        }

        fn mutate<R: Rng>(&mut self, rng: &mut R, _rate: f64) {
            self.value = rng.gen_range(0.0..1.0);
        }

        fn crossover<R: Rng>(&self, other: &Self, _rng: &mut R) -> Self {
            Self {
                value: (self.value + other.value) / 2.0,
            }
        }

        fn to_phenotype(&self) -> f64 {
            self.value
        }
    }

    #[test]
    fn test_tournament_new() {
        let tournament = Tournament::new(5);
        assert_eq!(tournament.size, 5);
    }

    #[test]
    fn test_tournament_selects_from_population() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 10,
            elitism: 1,
        };
        let mut pop: Population<TestGenome> = Population::random(config, &mut rng);

        // Assign fitness values
        for (i, ind) in pop.individuals.iter_mut().enumerate() {
            ind.fitness = Some(FitnessValue::Single(i as f64 / 10.0));
        }

        let tournament = Tournament::new(3);
        let selected = tournament.select(&pop, &mut rng);

        // Should select one of the individuals
        assert!(selected.fitness.is_some());
    }

    #[test]
    fn test_tournament_prefers_higher_fitness() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 10,
            elitism: 1,
        };
        let mut pop: Population<TestGenome> = Population::random(config, &mut rng);

        // Give one individual very high fitness
        for (i, ind) in pop.individuals.iter_mut().enumerate() {
            if i == 5 {
                ind.fitness = Some(FitnessValue::Single(100.0));
            } else {
                ind.fitness = Some(FitnessValue::Single(0.0));
            }
        }

        // With large tournament, should frequently select the best
        let tournament = Tournament::new(5); // Half population
        let mut high_fitness_count = 0;
        for _ in 0..100 {
            let selected = tournament.select(&pop, &mut rng);
            if selected.fitness_value() > 50.0 {
                high_fitness_count += 1;
            }
        }

        // With tournament size 5 out of 10, probability of including the best
        // in each tournament is 1 - (9/10)^5 ≈ 0.41, so we expect ~40+ out of 100
        assert!(high_fitness_count > 30, "Expected >30, got {}", high_fitness_count);
    }

    #[test]
    fn test_tournament_size_one_is_random() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 10,
            elitism: 1,
        };
        let mut pop: Population<TestGenome> = Population::random(config, &mut rng);

        for (i, ind) in pop.individuals.iter_mut().enumerate() {
            ind.fitness = Some(FitnessValue::Single(i as f64));
        }

        // Tournament size 1 = random selection
        let tournament = Tournament::new(1);
        let mut selections = std::collections::HashMap::new();

        for _ in 0..1000 {
            let selected = tournament.select(&pop, &mut rng);
            let fitness = selected.fitness_value() as i32;
            *selections.entry(fitness).or_insert(0) += 1;
        }

        // Should have selected multiple different individuals
        assert!(selections.len() > 1);
    }
}
