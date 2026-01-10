//! Selection operators.

use crate::genome::{BehaviorDescriptor, Genome};
use crate::population::{Individual, Population};
use rand::Rng;
use std::sync::{Arc, RwLock};

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

/// Archive for tracking explored behaviors (for novelty computation).
#[derive(Clone)]
pub struct NoveltyArchive {
    /// Stored behavior descriptors.
    behaviors: Vec<BehaviorDescriptor>,
    /// Maximum archive size.
    max_size: usize,
    /// Minimum novelty threshold for adding to archive.
    add_threshold: f64,
}

impl NoveltyArchive {
    /// Create a new novelty archive.
    pub fn new(max_size: usize, add_threshold: f64) -> Self {
        Self {
            behaviors: Vec::new(),
            max_size,
            add_threshold,
        }
    }

    /// Add a behavior to the archive if it's novel enough.
    pub fn add(&mut self, behavior: &BehaviorDescriptor, novelty: f64) -> bool {
        if novelty >= self.add_threshold && self.behaviors.len() < self.max_size {
            self.behaviors.push(behavior.clone());
            true
        } else {
            false
        }
    }

    /// Force add a behavior (ignoring threshold).
    pub fn force_add(&mut self, behavior: BehaviorDescriptor) {
        if self.behaviors.len() < self.max_size {
            self.behaviors.push(behavior);
        }
    }

    /// Get all behaviors in the archive.
    pub fn behaviors(&self) -> &[BehaviorDescriptor] {
        &self.behaviors
    }

    /// Number of behaviors in archive.
    pub fn len(&self) -> usize {
        self.behaviors.len()
    }

    /// Check if archive is empty.
    pub fn is_empty(&self) -> bool {
        self.behaviors.is_empty()
    }

    /// Compute novelty score for a behavior.
    /// Novelty = average distance to k nearest neighbors.
    pub fn compute_novelty(&self, behavior: &BehaviorDescriptor, k: usize, population_behaviors: &[&BehaviorDescriptor]) -> f64 {
        // Combine archive behaviors with current population
        let all_behaviors: Vec<&BehaviorDescriptor> = self.behaviors
            .iter()
            .chain(population_behaviors.iter().copied())
            .collect();

        if all_behaviors.is_empty() {
            return f64::MAX; // Maximum novelty if no comparisons
        }

        // Compute distances to all other behaviors
        let mut distances: Vec<f64> = all_behaviors
            .iter()
            .map(|other| behavior.distance(other))
            .filter(|d| *d > 0.0) // Exclude self
            .collect();

        if distances.is_empty() {
            return 0.0;
        }

        // Sort and take k nearest
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let k = k.min(distances.len());

        // Average distance to k nearest neighbors
        distances.iter().take(k).sum::<f64>() / k as f64
    }
}

impl Default for NoveltyArchive {
    fn default() -> Self {
        Self::new(1000, 0.1)
    }
}

/// Novelty-based selection.
/// Selects individuals based on how different they are from others.
pub struct NoveltySelection {
    /// Number of nearest neighbors for novelty computation.
    pub k: usize,
    /// Tournament size for selection.
    pub tournament_size: usize,
    /// Shared novelty archive.
    archive: Arc<RwLock<NoveltyArchive>>,
}

impl NoveltySelection {
    /// Create a new novelty selector.
    pub fn new(k: usize, tournament_size: usize, archive: Arc<RwLock<NoveltyArchive>>) -> Self {
        Self {
            k,
            tournament_size,
            archive,
        }
    }

    /// Create with default parameters (k=15, tournament=5).
    pub fn with_archive(archive: Arc<RwLock<NoveltyArchive>>) -> Self {
        Self::new(15, 5, archive)
    }

    /// Compute novelty scores for all individuals in population.
    pub fn compute_novelty_scores<G: Genome>(&self, population: &Population<G>) -> Vec<f64> {
        let archive = self.archive.read().unwrap();

        // Collect behaviors from population
        let pop_behaviors: Vec<&BehaviorDescriptor> = population
            .individuals
            .iter()
            .filter_map(|ind| ind.behavior.as_ref())
            .collect();

        // Compute novelty for each individual
        population
            .individuals
            .iter()
            .map(|ind| {
                ind.behavior
                    .as_ref()
                    .map(|b| archive.compute_novelty(b, self.k, &pop_behaviors))
                    .unwrap_or(0.0)
            })
            .collect()
    }

    /// Update the archive with novel individuals from the population.
    pub fn update_archive<G: Genome>(&self, population: &Population<G>) {
        let novelty_scores = self.compute_novelty_scores(population);
        let mut archive = self.archive.write().unwrap();

        for (ind, novelty) in population.individuals.iter().zip(novelty_scores.iter()) {
            if let Some(behavior) = &ind.behavior {
                archive.add(behavior, *novelty);
            }
        }
    }

    /// Get a reference to the archive.
    pub fn archive(&self) -> Arc<RwLock<NoveltyArchive>> {
        Arc::clone(&self.archive)
    }
}

impl<G: Genome> Selection<G> for NoveltySelection {
    fn select<'a, R: Rng>(
        &self,
        population: &'a Population<G>,
        rng: &mut R,
    ) -> &'a Individual<G> {
        let novelty_scores = self.compute_novelty_scores(population);
        let n = population.individuals.len();

        let mut best_idx = 0;
        let mut best_novelty = f64::NEG_INFINITY;

        // Tournament selection based on novelty
        for _ in 0..self.tournament_size {
            let idx = rng.gen_range(0..n);
            let novelty = novelty_scores[idx];

            if novelty > best_novelty {
                best_novelty = novelty;
                best_idx = idx;
            }
        }

        &population.individuals[best_idx]
    }
}

/// Combined fitness and novelty selection.
/// Uses a weighted combination of fitness and novelty scores.
pub struct NoveltyFitnessSelection {
    /// Novelty selector.
    novelty: NoveltySelection,
    /// Weight for fitness (0.0 = pure novelty, 1.0 = pure fitness).
    fitness_weight: f64,
}

impl NoveltyFitnessSelection {
    /// Create a new combined selector.
    pub fn new(novelty: NoveltySelection, fitness_weight: f64) -> Self {
        Self {
            novelty,
            fitness_weight: fitness_weight.clamp(0.0, 1.0),
        }
    }

    /// Compute combined scores (fitness + novelty).
    pub fn compute_combined_scores<G: Genome>(&self, population: &Population<G>) -> Vec<f64> {
        let novelty_scores = self.novelty.compute_novelty_scores(population);

        // Normalize novelty scores
        let max_novelty = novelty_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_novelty = novelty_scores.iter().cloned().fold(f64::INFINITY, f64::min);
        let novelty_range = max_novelty - min_novelty;

        population
            .individuals
            .iter()
            .zip(novelty_scores.iter())
            .map(|(ind, &novelty)| {
                let fitness = ind.fitness_value();
                let norm_novelty = if novelty_range > 0.0 {
                    (novelty - min_novelty) / novelty_range
                } else {
                    0.5
                };

                self.fitness_weight * fitness + (1.0 - self.fitness_weight) * norm_novelty
            })
            .collect()
    }

    /// Get a reference to the novelty archive.
    pub fn archive(&self) -> Arc<RwLock<NoveltyArchive>> {
        self.novelty.archive()
    }

    /// Update the archive with novel individuals.
    pub fn update_archive<G: Genome>(&self, population: &Population<G>) {
        self.novelty.update_archive(population);
    }
}

impl<G: Genome> Selection<G> for NoveltyFitnessSelection {
    fn select<'a, R: Rng>(
        &self,
        population: &'a Population<G>,
        rng: &mut R,
    ) -> &'a Individual<G> {
        let combined_scores = self.compute_combined_scores(population);
        let n = population.individuals.len();

        let mut best_idx = 0;
        let mut best_score = f64::NEG_INFINITY;

        // Tournament selection based on combined score
        for _ in 0..self.novelty.tournament_size {
            let idx = rng.gen_range(0..n);
            let score = combined_scores[idx];

            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        &population.individuals[best_idx]
    }
}

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

    // Novelty selection tests

    #[test]
    fn test_novelty_archive_new() {
        let archive = NoveltyArchive::new(100, 0.5);
        assert!(archive.is_empty());
        assert_eq!(archive.len(), 0);
    }

    #[test]
    fn test_novelty_archive_add() {
        let mut archive = NoveltyArchive::new(100, 0.5);
        let behavior = BehaviorDescriptor::new(vec![1.0, 2.0, 3.0]);

        // Should add if novelty >= threshold
        assert!(archive.add(&behavior, 0.6));
        assert_eq!(archive.len(), 1);

        // Should not add if novelty < threshold
        let behavior2 = BehaviorDescriptor::new(vec![4.0, 5.0, 6.0]);
        assert!(!archive.add(&behavior2, 0.3));
        assert_eq!(archive.len(), 1);
    }

    #[test]
    fn test_novelty_archive_force_add() {
        let mut archive = NoveltyArchive::new(100, 0.5);
        let behavior = BehaviorDescriptor::new(vec![1.0, 2.0, 3.0]);

        archive.force_add(behavior);
        assert_eq!(archive.len(), 1);
    }

    #[test]
    fn test_novelty_archive_compute_novelty() {
        let mut archive = NoveltyArchive::new(100, 0.0);

        // Add some behaviors
        archive.force_add(BehaviorDescriptor::new(vec![0.0, 0.0]));
        archive.force_add(BehaviorDescriptor::new(vec![1.0, 0.0]));
        archive.force_add(BehaviorDescriptor::new(vec![0.0, 1.0]));

        // Test novelty of a point close to origin
        let close_behavior = BehaviorDescriptor::new(vec![0.1, 0.1]);
        let novelty = archive.compute_novelty(&close_behavior, 2, &[]);
        assert!(novelty < 1.0, "Close point should have low novelty");

        // Test novelty of a point far from all others
        let far_behavior = BehaviorDescriptor::new(vec![10.0, 10.0]);
        let far_novelty = archive.compute_novelty(&far_behavior, 2, &[]);
        assert!(far_novelty > novelty, "Far point should have higher novelty");
    }

    #[test]
    fn test_novelty_selection_new() {
        let archive = Arc::new(RwLock::new(NoveltyArchive::default()));
        let selection = NoveltySelection::new(15, 5, archive);
        assert_eq!(selection.k, 15);
        assert_eq!(selection.tournament_size, 5);
    }

    #[test]
    fn test_novelty_selection_with_archive() {
        let archive = Arc::new(RwLock::new(NoveltyArchive::default()));
        let selection = NoveltySelection::with_archive(archive);
        assert_eq!(selection.k, 15);
        assert_eq!(selection.tournament_size, 5);
    }

    #[test]
    fn test_novelty_selection_select() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 10,
            elitism: 1,
        };
        let mut pop: Population<TestGenome> = Population::random(config, &mut rng);

        // Assign behaviors - make one very different
        for (i, ind) in pop.individuals.iter_mut().enumerate() {
            ind.fitness = Some(FitnessValue::Single(0.5));
            if i == 5 {
                ind.behavior = Some(BehaviorDescriptor::new(vec![100.0, 100.0]));
            } else {
                ind.behavior = Some(BehaviorDescriptor::new(vec![i as f64 * 0.1, i as f64 * 0.1]));
            }
        }

        let archive = Arc::new(RwLock::new(NoveltyArchive::default()));
        let selection = NoveltySelection::new(3, 5, archive);

        // Selection should work without panicking
        let selected = selection.select(&pop, &mut rng);
        assert!(selected.behavior.is_some());
    }

    #[test]
    fn test_novelty_selection_prefers_novel() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 10,
            elitism: 1,
        };
        let mut pop: Population<TestGenome> = Population::random(config, &mut rng);

        // Individuals clustered around origin with slight variation, except one outlier
        for (i, ind) in pop.individuals.iter_mut().enumerate() {
            ind.fitness = Some(FitnessValue::Single(0.5));
            if i == 5 {
                // Far outlier
                ind.behavior = Some(BehaviorDescriptor::new(vec![100.0, 100.0]));
            } else {
                // Slight variation around origin
                ind.behavior = Some(BehaviorDescriptor::new(vec![i as f64 * 0.01, i as f64 * 0.01]));
            }
        }

        let archive = Arc::new(RwLock::new(NoveltyArchive::default()));
        let selection = NoveltySelection::new(3, 8, archive);  // Large tournament

        // Count how often the novel individual is selected
        let mut novel_count = 0;
        for _ in 0..100 {
            let selected = selection.select(&pop, &mut rng);
            if let Some(behavior) = &selected.behavior {
                if behavior.values[0] > 50.0 {
                    novel_count += 1;
                }
            }
        }

        // Should frequently select the most novel individual
        // With tournament size 8 out of 10, probability of including outlier is very high
        assert!(novel_count > 30, "Expected >30, got {}", novel_count);
    }

    #[test]
    fn test_novelty_fitness_selection() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 10,
            elitism: 1,
        };
        let mut pop: Population<TestGenome> = Population::random(config, &mut rng);

        // Assign fitness and behaviors
        for (i, ind) in pop.individuals.iter_mut().enumerate() {
            ind.fitness = Some(FitnessValue::Single(i as f64 / 10.0));
            ind.behavior = Some(BehaviorDescriptor::new(vec![i as f64, i as f64]));
        }

        let archive = Arc::new(RwLock::new(NoveltyArchive::default()));
        let novelty = NoveltySelection::new(3, 5, archive);
        let selection = NoveltyFitnessSelection::new(novelty, 0.5);

        // Selection should work
        let selected = selection.select(&pop, &mut rng);
        assert!(selected.fitness.is_some());
    }

    #[test]
    fn test_novelty_archive_update() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let config = PopulationConfig {
            size: 5,
            elitism: 1,
        };
        let mut pop: Population<TestGenome> = Population::random(config, &mut rng);

        // Assign behaviors
        for (i, ind) in pop.individuals.iter_mut().enumerate() {
            ind.fitness = Some(FitnessValue::Single(0.5));
            ind.behavior = Some(BehaviorDescriptor::new(vec![i as f64 * 10.0, i as f64 * 10.0]));
        }

        let archive = Arc::new(RwLock::new(NoveltyArchive::new(100, 0.0)));
        let selection = NoveltySelection::new(3, 5, archive.clone());

        selection.update_archive(&pop);

        // Archive should have some behaviors now
        let archive_read = archive.read().unwrap();
        assert!(archive_read.len() > 0);
    }
}
