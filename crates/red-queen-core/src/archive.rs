//! Quality-Diversity archives.

use crate::genome::{BehaviorDescriptor, Genome};
use crate::population::Individual;
use rand::Rng;
use std::collections::HashMap;

/// Trait for QD archives.
pub trait Archive<G: Genome>: Send + Sync {
    /// Try to add an individual to the archive.
    /// Returns true if the individual was added.
    fn add(&mut self, individual: Individual<G>) -> bool;

    /// Get all archived individuals.
    fn get_all(&self) -> Vec<&Individual<G>>;

    /// Get coverage statistics.
    fn coverage(&self) -> ArchiveCoverage;

    /// Number of individuals in the archive.
    fn len(&self) -> usize;

    /// Check if archive is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Coverage statistics for an archive.
#[derive(Debug, Clone)]
pub struct ArchiveCoverage {
    /// Number of cells filled.
    pub filled: usize,
    /// Total number of cells.
    pub total: usize,
}

impl ArchiveCoverage {
    /// Coverage as a percentage.
    pub fn percentage(&self) -> f64 {
        self.filled as f64 / self.total as f64
    }
}

/// Behavior dimension configuration.
#[derive(Debug, Clone)]
pub struct BehaviorDimension {
    /// Name of the dimension.
    pub name: String,
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
    /// Number of bins.
    pub bins: usize,
}

/// MAP-Elites archive.
pub struct MapElitesArchive<G: Genome> {
    /// The grid of cells.
    grid: HashMap<Vec<usize>, Individual<G>>,
    /// Dimension configurations.
    dimensions: Vec<BehaviorDimension>,
}

impl<G: Genome> std::fmt::Debug for MapElitesArchive<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapElitesArchive")
            .field("cells_filled", &self.grid.len())
            .field("dimensions", &self.dimensions)
            .finish()
    }
}

impl<G: Genome> MapElitesArchive<G> {
    /// Create a new MAP-Elites archive.
    pub fn new(dimensions: Vec<BehaviorDimension>) -> Self {
        Self {
            grid: HashMap::new(),
            dimensions,
        }
    }

    /// Convert behavior to cell indices.
    fn behavior_to_cell(&self, behavior: &BehaviorDescriptor) -> Vec<usize> {
        self.dimensions
            .iter()
            .zip(behavior.values.iter())
            .map(|(dim, value)| {
                let normalized = (value - dim.min) / (dim.max - dim.min);
                let bin = (normalized * dim.bins as f64).floor() as usize;
                bin.min(dim.bins - 1)
            })
            .collect()
    }

    /// Get total number of cells.
    fn total_cells(&self) -> usize {
        self.dimensions.iter().map(|d| d.bins).product()
    }
}

impl<G: Genome> Archive<G> for MapElitesArchive<G> {
    fn add(&mut self, individual: Individual<G>) -> bool {
        let Some(behavior) = &individual.behavior else {
            return false;
        };

        let cell = self.behavior_to_cell(behavior);

        // Check if cell is empty or new individual is better
        let dominated = self
            .grid
            .get(&cell)
            .map(|existing| individual.fitness_value() > existing.fitness_value())
            .unwrap_or(true);

        if dominated {
            self.grid.insert(cell, individual);
            true
        } else {
            false
        }
    }

    fn get_all(&self) -> Vec<&Individual<G>> {
        self.grid.values().collect()
    }

    fn coverage(&self) -> ArchiveCoverage {
        ArchiveCoverage {
            filled: self.grid.len(),
            total: self.total_cells(),
        }
    }

    fn len(&self) -> usize {
        self.grid.len()
    }
}

impl<G: Genome> MapElitesArchive<G> {
    /// Select a random individual from the archive.
    pub fn select_random<R: Rng>(&self, rng: &mut R) -> Option<&Individual<G>> {
        if self.grid.is_empty() {
            return None;
        }
        let idx = rng.gen_range(0..self.grid.len());
        self.grid.values().nth(idx)
    }

    /// Get the best individual across all cells.
    pub fn best(&self) -> Option<&Individual<G>> {
        self.grid
            .values()
            .max_by(|a, b| {
                a.fitness_value()
                    .partial_cmp(&b.fitness_value())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get top N individuals across all cells.
    pub fn top_n(&self, n: usize) -> Vec<&Individual<G>> {
        let mut individuals: Vec<_> = self.grid.values().collect();
        individuals.sort_by(|a, b| {
            b.fitness_value()
                .partial_cmp(&a.fitness_value())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        individuals.truncate(n);
        individuals
    }

    /// Get the dimension configurations.
    pub fn dimensions(&self) -> &[BehaviorDimension] {
        &self.dimensions
    }

    /// Get cell contents for visualization/analysis.
    pub fn get_cell(&self, indices: &[usize]) -> Option<&Individual<G>> {
        self.grid.get(indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fitness::FitnessValue;
    use rand::Rng;

    // Simple test genome
    #[derive(Clone, Debug)]
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

    fn make_individual(value: f64, fitness: f64, behavior: Vec<f64>) -> Individual<TestGenome> {
        Individual {
            genome: TestGenome { value },
            fitness: Some(FitnessValue::Single(fitness)),
            behavior: Some(BehaviorDescriptor::new(behavior)),
            birth_generation: 0,
        }
    }

    #[test]
    fn test_archive_coverage_percentage() {
        let coverage = ArchiveCoverage {
            filled: 25,
            total: 100,
        };
        assert!((coverage.percentage() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_map_elites_empty() {
        let dimensions = vec![
            BehaviorDimension {
                name: "dim1".to_string(),
                min: 0.0,
                max: 1.0,
                bins: 10,
            },
            BehaviorDimension {
                name: "dim2".to_string(),
                min: 0.0,
                max: 1.0,
                bins: 10,
            },
        ];
        let archive: MapElitesArchive<TestGenome> = MapElitesArchive::new(dimensions);

        assert_eq!(archive.get_all().len(), 0);
        let coverage = archive.coverage();
        assert_eq!(coverage.filled, 0);
        assert_eq!(coverage.total, 100); // 10 * 10
    }

    #[test]
    fn test_map_elites_add_individual() {
        let dimensions = vec![BehaviorDimension {
            name: "dim1".to_string(),
            min: 0.0,
            max: 1.0,
            bins: 10,
        }];
        let mut archive: MapElitesArchive<TestGenome> = MapElitesArchive::new(dimensions);

        let ind = make_individual(0.5, 1.0, vec![0.5]);
        assert!(archive.add(ind));

        assert_eq!(archive.get_all().len(), 1);
        assert_eq!(archive.coverage().filled, 1);
    }

    #[test]
    fn test_map_elites_reject_no_behavior() {
        let dimensions = vec![BehaviorDimension {
            name: "dim1".to_string(),
            min: 0.0,
            max: 1.0,
            bins: 10,
        }];
        let mut archive: MapElitesArchive<TestGenome> = MapElitesArchive::new(dimensions);

        let ind = Individual {
            genome: TestGenome { value: 0.5 },
            fitness: Some(FitnessValue::Single(1.0)),
            behavior: None, // No behavior
            birth_generation: 0,
        };

        assert!(!archive.add(ind));
        assert_eq!(archive.get_all().len(), 0);
    }

    #[test]
    fn test_map_elites_replace_with_better() {
        let dimensions = vec![BehaviorDimension {
            name: "dim1".to_string(),
            min: 0.0,
            max: 1.0,
            bins: 10,
        }];
        let mut archive: MapElitesArchive<TestGenome> = MapElitesArchive::new(dimensions);

        // Add first individual
        let ind1 = make_individual(0.5, 0.3, vec![0.55]);
        assert!(archive.add(ind1));

        // Add better individual to same cell
        let ind2 = make_individual(0.6, 0.8, vec![0.55]);
        assert!(archive.add(ind2));

        // Should still have 1 individual (replaced)
        assert_eq!(archive.get_all().len(), 1);
        assert!((archive.get_all()[0].fitness_value() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_map_elites_reject_worse() {
        let dimensions = vec![BehaviorDimension {
            name: "dim1".to_string(),
            min: 0.0,
            max: 1.0,
            bins: 10,
        }];
        let mut archive: MapElitesArchive<TestGenome> = MapElitesArchive::new(dimensions);

        // Add first individual with high fitness
        let ind1 = make_individual(0.5, 0.9, vec![0.55]);
        assert!(archive.add(ind1));

        // Try to add worse individual to same cell
        let ind2 = make_individual(0.6, 0.3, vec![0.55]);
        assert!(!archive.add(ind2));

        // Should still have original
        assert!((archive.get_all()[0].fitness_value() - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_map_elites_different_cells() {
        let dimensions = vec![BehaviorDimension {
            name: "dim1".to_string(),
            min: 0.0,
            max: 1.0,
            bins: 10,
        }];
        let mut archive: MapElitesArchive<TestGenome> = MapElitesArchive::new(dimensions);

        // Add to different cells
        let ind1 = make_individual(0.1, 0.5, vec![0.15]); // bin 1
        let ind2 = make_individual(0.5, 0.5, vec![0.55]); // bin 5
        let ind3 = make_individual(0.9, 0.5, vec![0.95]); // bin 9

        assert!(archive.add(ind1));
        assert!(archive.add(ind2));
        assert!(archive.add(ind3));

        assert_eq!(archive.get_all().len(), 3);
        assert_eq!(archive.coverage().filled, 3);
    }

    #[test]
    fn test_map_elites_2d_grid() {
        let dimensions = vec![
            BehaviorDimension {
                name: "x".to_string(),
                min: 0.0,
                max: 1.0,
                bins: 5,
            },
            BehaviorDimension {
                name: "y".to_string(),
                min: 0.0,
                max: 1.0,
                bins: 5,
            },
        ];
        let mut archive: MapElitesArchive<TestGenome> = MapElitesArchive::new(dimensions);

        // Add to corner cells
        archive.add(make_individual(0.1, 0.5, vec![0.1, 0.1])); // (0,0)
        archive.add(make_individual(0.2, 0.5, vec![0.9, 0.1])); // (4,0)
        archive.add(make_individual(0.3, 0.5, vec![0.1, 0.9])); // (0,4)
        archive.add(make_individual(0.4, 0.5, vec![0.9, 0.9])); // (4,4)

        assert_eq!(archive.coverage().total, 25); // 5 * 5
        assert_eq!(archive.coverage().filled, 4);
    }
}
