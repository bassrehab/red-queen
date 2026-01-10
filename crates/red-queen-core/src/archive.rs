//! Quality-Diversity archives.

use crate::genome::{BehaviorDescriptor, Genome};
use crate::population::Individual;
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
}
