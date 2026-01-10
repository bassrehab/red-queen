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
        // Collect unevaluated individuals
        let unevaluated: Vec<_> = self
            .population
            .individuals
            .iter()
            .enumerate()
            .filter(|(_, ind)| ind.fitness.is_none())
            .map(|(i, ind)| (i, ind.genome.clone()))
            .collect();

        if unevaluated.is_empty() {
            return;
        }

        // Batch evaluate
        let genomes: Vec<_> = unevaluated.iter().map(|(_, g)| g.clone()).collect();
        let results = self.fitness.evaluate_batch(&genomes);

        // Apply results
        for ((idx, _), result) in unevaluated.into_iter().zip(results) {
            self.population.individuals[idx].fitness = Some(result.value);
            self.population.individuals[idx].behavior = result.behavior;
        }
    }

    fn evaluate_batch(&self, mut individuals: Vec<Individual<G>>) -> Vec<Individual<G>> {
        if individuals.is_empty() {
            return individuals;
        }

        // Batch evaluate all genomes
        let genomes: Vec<_> = individuals.iter().map(|ind| ind.genome.clone()).collect();
        let results = self.fitness.evaluate_batch(&genomes);

        // Apply results
        for (ind, result) in individuals.iter_mut().zip(results) {
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

use crate::archive::MapElitesArchive;

/// Configuration for MAP-Elites evolution.
#[derive(Debug, Clone)]
pub struct MapElitesConfig {
    /// Number of offspring to generate per generation (batch size).
    pub batch_size: usize,
    /// Variation configuration.
    pub variation: VariationConfig,
    /// Random seed (None for random).
    pub seed: Option<u64>,
    /// Number of random individuals to seed the archive with.
    pub initial_population: usize,
}

impl Default for MapElitesConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            variation: VariationConfig::default(),
            seed: None,
            initial_population: 100,
        }
    }
}

/// Results from MAP-Elites evolution.
#[derive(Debug)]
pub struct MapElitesResult<G: Genome> {
    /// The final archive.
    pub archive: MapElitesArchive<G>,
    /// Best individual found (highest fitness).
    pub best: Option<Individual<G>>,
    /// Number of generations run.
    pub generations: usize,
    /// Total individuals evaluated.
    pub total_evaluations: usize,
}

/// MAP-Elites evolution engine.
///
/// Unlike traditional evolution, MAP-Elites maintains a grid of cells
/// where each cell can hold at most one individual. Individuals compete
/// for cells based on their behavior descriptor, and only the fittest
/// individual in each cell survives.
pub struct MapElitesEvolution<G, F>
where
    G: Genome,
    F: Fitness<G>,
{
    archive: MapElitesArchive<G>,
    fitness: F,
    config: MapElitesConfig,
    rng: ChaCha8Rng,
    total_evaluations: usize,
}

impl<G, F> MapElitesEvolution<G, F>
where
    G: Genome,
    F: Fitness<G>,
{
    /// Create a new MAP-Elites evolution instance.
    pub fn new(archive: MapElitesArchive<G>, fitness: F, config: MapElitesConfig) -> Self {
        let seed = config.seed.unwrap_or_else(rand::random);
        let rng = ChaCha8Rng::seed_from_u64(seed);

        Self {
            archive,
            fitness,
            config,
            rng,
            total_evaluations: 0,
        }
    }

    /// Seed the archive with random individuals.
    pub fn initialize(&mut self) {
        // Generate all random genomes
        let genomes: Vec<G> = (0..self.config.initial_population)
            .map(|_| G::random(&mut self.rng))
            .collect();

        // Batch evaluate
        let results = self.fitness.evaluate_batch(&genomes);

        // Add to archive
        for (genome, result) in genomes.into_iter().zip(results) {
            let mut individual = Individual::new(genome);
            individual.fitness = Some(result.value);
            individual.behavior = result.behavior;
            self.archive.add(individual);
            self.total_evaluations += 1;
        }
    }

    /// Run MAP-Elites for a number of generations.
    pub fn run(&mut self, generations: usize) -> MapElitesResult<G> {
        // Initialize if archive is empty
        if self.archive.is_empty() {
            self.initialize();
        }

        for _gen in 0..generations {
            self.step();
        }

        let best = self.archive.best().cloned();
        let dimensions = self.archive.dimensions().to_vec();
        let archive = std::mem::replace(&mut self.archive, MapElitesArchive::new(dimensions));

        MapElitesResult {
            archive,
            best,
            generations,
            total_evaluations: self.total_evaluations,
        }
    }

    /// Run a single generation step.
    pub fn step(&mut self) {
        let offspring = self.create_offspring();

        // Batch evaluate all offspring
        let genomes: Vec<_> = offspring.iter().map(|ind| ind.genome.clone()).collect();
        let results = self.fitness.evaluate_batch(&genomes);

        // Add evaluated individuals to archive
        for (mut ind, result) in offspring.into_iter().zip(results) {
            ind.fitness = Some(result.value);
            ind.behavior = result.behavior;
            self.archive.add(ind);
            self.total_evaluations += 1;
        }
    }

    fn create_offspring(&mut self) -> Vec<Individual<G>> {
        let mut offspring = Vec::with_capacity(self.config.batch_size);

        for _ in 0..self.config.batch_size {
            let child_genome = if self.archive.len() < 2 {
                // Not enough individuals for crossover, create random
                G::random(&mut self.rng)
            } else {
                // Select two parents from the archive
                let parent1 = self.archive.select_random(&mut self.rng);
                let parent2 = self.archive.select_random(&mut self.rng);

                match (parent1, parent2) {
                    (Some(p1), Some(p2)) => {
                        vary(
                            &p1.genome,
                            &p2.genome,
                            &self.config.variation,
                            &mut self.rng,
                        )
                    }
                    _ => G::random(&mut self.rng),
                }
            };

            offspring.push(Individual::new(child_genome));
        }

        offspring
    }

    /// Get a reference to the current archive.
    pub fn archive(&self) -> &MapElitesArchive<G> {
        &self.archive
    }

    /// Get the total number of evaluations performed.
    pub fn total_evaluations(&self) -> usize {
        self.total_evaluations
    }

    /// Get coverage statistics.
    pub fn coverage(&self) -> crate::archive::ArchiveCoverage {
        self.archive.coverage()
    }
}
