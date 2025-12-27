//! Simple evolution example.
//!
//! This example demonstrates a basic evolutionary run using:
//! - Random population of LLMAttackGenome
//! - MockTarget that always refuses
//! - Tournament selection
//! - 10 generations of evolution
//!
//! Run with: `cargo run --example simple_evolution`

use std::sync::Arc;

use red_queen_core::evolution::{Evolution, EvolutionConfig};
use red_queen_core::population::PopulationConfig;
use red_queen_core::selection::Tournament;
use red_queen_core::variation::VariationConfig;
use red_queen_llm::fitness::JailbreakFitness;
use red_queen_llm::genome::LLMAttackGenome;
use red_queen_llm::targets::MockTarget;

fn main() {
    println!("=== Red Queen: Simple Evolution Example ===\n");

    // Create a mock target that always refuses
    let target = Arc::new(MockTarget::always_refuse());
    println!("Target: MockTarget (always refuses)");

    // Create fitness evaluator
    let fitness = JailbreakFitness::new(target);

    // Create tournament selection with size 3
    let selection = Tournament::new(3);
    println!("Selection: Tournament (size 3)");

    // Configure evolution
    let config = EvolutionConfig {
        population: PopulationConfig {
            size: 20,
            elitism: 2,
        },
        variation: VariationConfig {
            mutation_rate: 0.3,
            crossover_rate: 0.7,
        },
        seed: Some(42), // Fixed seed for reproducibility
    };
    println!("Population size: {}", config.population.size);
    println!("Elitism: {}", config.population.elitism);
    println!("Mutation rate: {}", config.variation.mutation_rate);
    println!("Crossover rate: {}", config.variation.crossover_rate);
    println!();

    // Create and run evolution
    let mut evolution: Evolution<LLMAttackGenome, _, _> =
        Evolution::new(fitness, selection, config);

    println!("Running 10 generations...\n");
    let result = evolution.run(10);

    // Print results
    println!("=== Results ===\n");
    println!("Generations completed: {}", result.generations);
    println!("Final population size: {}", result.population.individuals.len());
    println!();

    // Print best individual
    println!("Best attack found:");
    println!("  Strategy: {:?}", result.best.genome.primary_strategy);
    println!("  Encoding: {:?}", result.best.genome.encoding);
    println!(
        "  Secondary strategy: {:?}",
        result.best.genome.secondary_strategy
    );
    println!(
        "  Fitness: {:.2}",
        result.best.fitness.as_ref().map(|f| f.primary()).unwrap_or(0.0)
    );
    println!();
    println!("Generated prompt:");
    println!("---");
    println!("{}", result.best.genome.to_prompt());
    println!("---");
    println!();

    // Print top 5 individuals
    println!("Top 5 individuals:");
    for (i, ind) in result.population.top_n(5).iter().enumerate() {
        println!(
            "  {}. {:?} / {:?} - fitness: {:.2}",
            i + 1,
            ind.genome.primary_strategy,
            ind.genome.encoding,
            ind.fitness_value()
        );
    }

    println!("\nEvolution complete!");
}
