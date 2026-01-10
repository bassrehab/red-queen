//! # Red Queen Core
//!
//! Core evolutionary computation engine for adversarial testing.
//!
//! This crate provides the foundational building blocks for evolutionary algorithms:
//! - Population management
//! - Genome and Fitness traits
//! - Selection operators (Tournament, Lexicase, Novelty)
//! - Variation operators (Mutation, Crossover)
//! - Quality-Diversity archives (MAP-Elites)
//!
//! ## Example
//!
//! ```rust,ignore
//! use red_queen_core::prelude::*;
//!
//! let evolution = Evolution::builder()
//!     .with_population_size(100)
//!     .with_selection(Tournament::new(3))
//!     .build();
//!
//! let results = evolution.run(500);
//! ```

pub mod genome;
pub mod fitness;
pub mod population;
pub mod selection;
pub mod variation;
pub mod archive;
pub mod evolution;
pub mod error;

pub mod prelude {
    //! Convenient re-exports for common usage.
    pub use crate::genome::Genome;
    pub use crate::fitness::{Fitness, FitnessValue};
    pub use crate::population::{Population, Individual};
    pub use crate::selection::{Selection, Tournament, NoveltyArchive, NoveltySelection, NoveltyFitnessSelection};
    pub use crate::evolution::{Evolution, MapElitesEvolution, MapElitesConfig, MapElitesResult};
    pub use crate::archive::{Archive, MapElitesArchive, BehaviorDimension, ArchiveCoverage};
    pub use crate::error::{Error, Result};
}
