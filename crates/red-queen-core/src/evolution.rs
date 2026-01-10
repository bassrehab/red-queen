//! Evolution engine.

use crate::archive::Archive;
use crate::fitness::Fitness;
use crate::genome::Genome;
use crate::population::{Individual, Population, PopulationConfig};
use crate::selection::Selection;
use crate::variation::{vary, VariationConfig};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Evolution configuration.
#[derive(Debug, Clone)]
pub struct EvolutionConfig {
    /// Population configuration.
    pub population: PopulationConfig,
    /// Variation configuration.
    pub variation: VariationConfig,
    /// Random seed (None for random).
    pub seed: Option<u64>,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            population: PopulationConfig::default(),
            variation: VariationConfig::default(),
            seed: None,
        }
    }
}

/// Results from evolution.
#[derive(Debug)]
pub struct EvolutionResult<G: Genome> {
    /// Final population.
    pub population: Population<G>,
    /// Best individual found.
    pub best: Individual<G>,
    /// Number of generations run.
    pub generations: usize,
}

/// The main evolution engine.
pub struct Evolution<G, F, S>
where
    G: Genome,
    F: Fitness<G>,
    S: Selection<G>,
{
    population: Population<G>,
    fitness: F,
    selection: S,
    archive: Option<Box<dyn Archive<G>>>,
    config: EvolutionConfig,
    rng: ChaCha8Rng,
}

impl<G, F, S> Evolution<G, F, S>
where
    G: Genome,
    F: Fitness<G>,
    S: Selection<G>,
{
    /// Create a new evolution instance.
    pub fn new(fitness: F, selection: S, config: EvolutionConfig) -> Self {
        let seed = config.seed.unwrap_or_else(rand::random);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let population = Population::random(config.population.clone(), &mut rng);

        Self {
            population,
            fitness,
            selection,
            archive: None,
            config,
            rng,
        }
    }

    /// Set the QD archive.
    pub fn with_archive(mut self, archive: Box<dyn Archive<G>>) -> Self {
        self.archive = Some(archive);
        self
    }

    /// Run evolution for a number of generations.
    pub fn run(&mut self, generations: usize) -> EvolutionResult<G> {
        // Initial evaluation
        self.evaluate_population();

        for _gen in 0..generations {
            // Create offspring
            let offspring = self.create_offspring();

            // Evaluate offspring
            let evaluated = self.evaluate_batch(offspring);

            // Update archive
            if let Some(archive) = &mut self.archive {
                for ind in &evaluated {
                    archive.add(ind.clone());
                }
            }

            // Survivor selection
            self.select_survivors(evaluated);

            self.population.generation += 1;
        }

        let best = self.population.best().cloned().unwrap_or_else(|| {
            Individual::new(G::random(&mut self.rng))
        });

        EvolutionResult {
            population: self.population.clone(),
            best,
            generations,
        }
    }

    fn evaluate_population(&mut self) {
        for ind in &mut self.population.individuals {
            if ind.fitness.is_none() {
                let result = self.fitness.evaluate(&ind.genome);
                ind.fitness = Some(result.value);
                ind.behavior = result.behavior;
            }
        }
    }

    fn evaluate_batch(&self, mut individuals: Vec<Individual<G>>) -> Vec<Individual<G>> {
        for ind in &mut individuals {
            let result = self.fitness.evaluate(&ind.genome);
            ind.fitness = Some(result.value);
            ind.behavior = result.behavior;
        }
        individuals
    }

    fn create_offspring(&mut self) -> Vec<Individual<G>> {
        let count = self.config.population.size - self.config.population.elitism;

        (0..count)
            .map(|_| {
                let parent1 = self.selection.select(&self.population, &mut self.rng);
                let parent2 = self.selection.select(&self.population, &mut self.rng);

                let genome = vary(
                    &parent1.genome,
                    &parent2.genome,
                    &self.config.variation,
                    &mut self.rng,
                );

                Individual {
                    genome,
                    fitness: None,
                    behavior: None,
                    birth_generation: self.population.generation + 1,
                }
            })
            .collect()
    }

    fn select_survivors(&mut self, offspring: Vec<Individual<G>>) {
        // Keep elites
        let elites: Vec<_> = self
            .population
            .top_n(self.config.population.elitism)
            .into_iter()
            .cloned()
            .collect();

        // Combine elites and offspring
        let mut new_pop = elites;
        new_pop.extend(offspring);

        // Truncate to population size
        new_pop.truncate(self.config.population.size);

        self.population.individuals = new_pop;
    }
}

// Need to add rand_chacha to dependencies
use rand_chacha;
