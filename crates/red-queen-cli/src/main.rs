//! Red Queen CLI
//!
//! Command-line interface for running evolutionary adversarial testing.

use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};

use red_queen_core::evolution::{Evolution, EvolutionConfig};
use red_queen_core::population::PopulationConfig;
use red_queen_core::selection::Tournament;
use red_queen_core::variation::VariationConfig;
use red_queen_llm::fitness::JailbreakFitness;
use red_queen_llm::genome::LLMAttackGenome;
use red_queen_llm::targets::{AnthropicTarget, GeminiTarget, LLMTarget, MockTarget, OpenAITarget};

#[derive(Parser)]
#[command(name = "red-queen")]
#[command(about = "Evolutionary adversarial testing framework")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// LLM safety domain commands
    Llm {
        #[command(subcommand)]
        action: LlmCommands,
    },
    /// Analyze results from a previous run
    Analyze {
        /// Path to results file
        #[arg(short, long)]
        input: String,
    },
}

#[derive(Subcommand)]
enum LlmCommands {
    /// Discover jailbreaks through evolution
    Jailbreak {
        /// Target model. Formats:
        ///   openai:gpt-4             - OpenAI GPT-4
        ///   anthropic:claude-sonnet-4-20250514 - Anthropic Claude
        ///   gemini:gemini-2.0-flash  - Google Gemini
        ///   mock:refuse              - Mock target (always refuses)
        ///   mock:comply              - Mock target (always complies)
        #[arg(short, long)]
        target: String,

        /// Number of generations to evolve
        #[arg(short, long, default_value = "100")]
        generations: usize,

        /// Population size
        #[arg(short, long, default_value = "50")]
        population: usize,

        /// Tournament selection size
        #[arg(long, default_value = "3")]
        tournament_size: usize,

        /// Mutation rate (0.0 to 1.0)
        #[arg(long, default_value = "0.3")]
        mutation_rate: f64,

        /// Crossover rate (0.0 to 1.0)
        #[arg(long, default_value = "0.7")]
        crossover_rate: f64,

        /// Random seed for reproducibility (optional)
        #[arg(long)]
        seed: Option<u64>,

        /// Output file for results
        #[arg(short, long, default_value = "results.json")]
        output: String,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

/// Parse target string into an LLMTarget.
fn parse_target(target_str: &str) -> Result<Arc<dyn LLMTarget>> {
    let parts: Vec<&str> = target_str.split(':').collect();

    match parts.as_slice() {
        ["mock", "refuse"] | ["mock"] => Ok(Arc::new(MockTarget::always_refuse())),
        ["mock", "comply"] => Ok(Arc::new(MockTarget::always_comply())),
        ["openai", model] => {
            let target = OpenAITarget::from_env(model)
                .context("Failed to create OpenAI target. Is OPENAI_API_KEY set?")?;
            Ok(Arc::new(target))
        }
        ["anthropic", model] => {
            let target = AnthropicTarget::from_env(model)
                .context("Failed to create Anthropic target. Is ANTHROPIC_API_KEY set?")?;
            Ok(Arc::new(target))
        }
        ["gemini", model] => {
            let target = GeminiTarget::from_env(model)
                .context("Failed to create Gemini target. Is GOOGLE_API_KEY set?")?;
            Ok(Arc::new(target))
        }
        _ => Err(anyhow!(
            "Unknown target: '{}'. Use 'openai:', 'anthropic:', 'gemini:', or 'mock:'",
            target_str
        )),
    }
}

/// Result entry for a single attack.
#[derive(Debug, Serialize, Deserialize)]
struct AttackResult {
    strategy: String,
    encoding: String,
    secondary_strategy: Option<String>,
    fitness: f64,
    prompt: String,
    generation: usize,
}

/// Full results from an evolution run.
#[derive(Debug, Serialize, Deserialize)]
struct EvolutionResults {
    target: String,
    generations: usize,
    population_size: usize,
    best_attack: AttackResult,
    top_attacks: Vec<AttackResult>,
    config: RunConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct RunConfig {
    mutation_rate: f64,
    crossover_rate: f64,
    tournament_size: usize,
    seed: Option<u64>,
}

fn genome_to_result(genome: &LLMAttackGenome, fitness: f64, generation: usize) -> AttackResult {
    AttackResult {
        strategy: format!("{:?}", genome.primary_strategy),
        encoding: format!("{:?}", genome.encoding),
        secondary_strategy: genome.secondary_strategy.as_ref().map(|s| format!("{:?}", s)),
        fitness,
        prompt: genome.to_prompt(),
        generation,
    }
}

fn run_jailbreak_evolution(
    target_str: &str,
    generations: usize,
    population: usize,
    tournament_size: usize,
    mutation_rate: f64,
    crossover_rate: f64,
    seed: Option<u64>,
    output: &str,
    verbose: bool,
) -> Result<()> {
    println!();
    println!("  Red Queen - LLM Jailbreak Discovery");
    println!("  ====================================");
    println!();

    // Parse target
    let target = parse_target(target_str)?;
    println!("  Target:       {}", target_str);
    println!("  Generations:  {}", generations);
    println!("  Population:   {}", population);
    println!("  Tournament:   {}", tournament_size);
    println!("  Mutation:     {:.1}%", mutation_rate * 100.0);
    println!("  Crossover:    {:.1}%", crossover_rate * 100.0);
    if let Some(s) = seed {
        println!("  Seed:         {}", s);
    }
    println!("  Output:       {}", output);
    println!();

    // Create fitness evaluator
    let fitness = JailbreakFitness::new(target);

    // Create selection
    let selection = Tournament::new(tournament_size);

    // Configure evolution
    let config = EvolutionConfig {
        population: PopulationConfig {
            size: population,
            elitism: (population / 10).max(1), // 10% elitism
        },
        variation: VariationConfig {
            mutation_rate,
            crossover_rate,
        },
        seed,
    };

    // Create evolution engine
    let mut evolution: Evolution<LLMAttackGenome, _, _> =
        Evolution::new(fitness, selection, config);

    // Set up progress bar
    let pb = ProgressBar::new(generations as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} generations")
            .unwrap()
            .progress_chars("#>-"),
    );

    println!("  Starting evolution...");
    println!();

    // Run evolution with progress updates
    // Note: Currently runs all at once, but structured for future per-generation hooks
    let result = evolution.run(generations);

    pb.finish_with_message("complete");
    println!();

    // Get results
    let best = &result.best;
    let best_fitness = best.fitness.as_ref().map(|f| f.primary()).unwrap_or(0.0);

    println!("  Results");
    println!("  -------");
    println!();
    println!("  Best fitness: {:.2}", best_fitness);
    println!("  Strategy:     {:?}", best.genome.primary_strategy);
    println!("  Encoding:     {:?}", best.genome.encoding);
    println!();

    if verbose {
        println!("  Best prompt:");
        println!("  ---");
        for line in best.genome.to_prompt().lines() {
            println!("  {}", line);
        }
        println!("  ---");
        println!();
    }

    // Collect top attacks
    let top_attacks: Vec<AttackResult> = result
        .population
        .top_n(10)
        .iter()
        .enumerate()
        .map(|(_i, ind)| {
            genome_to_result(
                &ind.genome,
                ind.fitness_value(),
                ind.birth_generation,
            )
        })
        .collect();

    println!("  Top 10 attacks:");
    for (i, attack) in top_attacks.iter().enumerate() {
        println!(
            "    {}. {:?} / {:?} - fitness: {:.2}",
            i + 1,
            attack.strategy,
            attack.encoding,
            attack.fitness
        );
    }
    println!();

    // Build results structure
    let results = EvolutionResults {
        target: target_str.to_string(),
        generations,
        population_size: population,
        best_attack: genome_to_result(&best.genome, best_fitness, best.birth_generation),
        top_attacks,
        config: RunConfig {
            mutation_rate,
            crossover_rate,
            tournament_size,
            seed,
        },
    };

    // Export to JSON
    let json = serde_json::to_string_pretty(&results)?;
    let mut file = File::create(output)?;
    file.write_all(json.as_bytes())?;
    println!("  Results saved to: {}", output);
    println!();

    Ok(())
}

fn analyze_results(input: &str) -> Result<()> {
    println!();
    println!("  Red Queen - Results Analysis");
    println!("  =============================");
    println!();

    // Read results file
    let content = std::fs::read_to_string(input)
        .with_context(|| format!("Failed to read file: {}", input))?;

    let results: EvolutionResults = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse results file: {}", input))?;

    println!("  Target:       {}", results.target);
    println!("  Generations:  {}", results.generations);
    println!("  Population:   {}", results.population_size);
    println!();
    println!("  Best Attack");
    println!("  -----------");
    println!("  Fitness:      {:.2}", results.best_attack.fitness);
    println!("  Strategy:     {}", results.best_attack.strategy);
    println!("  Encoding:     {}", results.best_attack.encoding);
    println!();
    println!("  Prompt:");
    println!("  ---");
    for line in results.best_attack.prompt.lines() {
        println!("  {}", line);
    }
    println!("  ---");
    println!();

    // Strategy distribution
    let mut strategy_counts: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    for attack in &results.top_attacks {
        *strategy_counts.entry(&attack.strategy).or_insert(0) += 1;
    }

    println!("  Strategy Distribution (Top 10):");
    for (strategy, count) in strategy_counts {
        println!("    {}: {}", strategy, count);
    }
    println!();

    Ok(())
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("red_queen=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Llm { action } => match action {
            LlmCommands::Jailbreak {
                target,
                generations,
                population,
                tournament_size,
                mutation_rate,
                crossover_rate,
                seed,
                output,
                verbose,
            } => {
                run_jailbreak_evolution(
                    &target,
                    generations,
                    population,
                    tournament_size,
                    mutation_rate,
                    crossover_rate,
                    seed,
                    &output,
                    verbose,
                )?;
            }
        },
        Commands::Analyze { input } => {
            analyze_results(&input)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target_mock_refuse() {
        let target = parse_target("mock:refuse").unwrap();
        assert_eq!(target.name(), "mock");
    }

    #[test]
    fn test_parse_target_mock_default() {
        let target = parse_target("mock").unwrap();
        assert_eq!(target.name(), "mock");
    }

    #[test]
    fn test_parse_target_mock_comply() {
        let target = parse_target("mock:comply").unwrap();
        assert_eq!(target.name(), "mock");
    }

    #[test]
    fn test_parse_target_openai_missing_key() {
        std::env::remove_var("OPENAI_API_KEY");
        let result = parse_target("openai:gpt-4");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("OPENAI_API_KEY"));
    }

    #[test]
    fn test_parse_target_anthropic_missing_key() {
        std::env::remove_var("ANTHROPIC_API_KEY");
        let result = parse_target("anthropic:claude-sonnet-4-20250514");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn test_parse_target_gemini_missing_key() {
        std::env::remove_var("GOOGLE_API_KEY");
        let result = parse_target("gemini:gemini-2.0-flash");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("GOOGLE_API_KEY"));
    }

    #[test]
    fn test_parse_target_unknown() {
        let result = parse_target("unknown:model");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("Unknown target"));
    }

    #[test]
    fn test_parse_target_invalid_format() {
        let result = parse_target("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_attack_result_serialization() {
        let result = AttackResult {
            strategy: "Roleplay".to_string(),
            encoding: "Base64".to_string(),
            secondary_strategy: Some("Encoding".to_string()),
            fitness: 0.75,
            prompt: "test prompt".to_string(),
            generation: 5,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Roleplay"));
        assert!(json.contains("0.75"));

        let parsed: AttackResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.strategy, "Roleplay");
        assert!((parsed.fitness - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_evolution_results_serialization() {
        let results = EvolutionResults {
            target: "mock:comply".to_string(),
            generations: 10,
            population_size: 20,
            best_attack: AttackResult {
                strategy: "DirectJailbreak".to_string(),
                encoding: "None".to_string(),
                secondary_strategy: None,
                fitness: 1.0,
                prompt: "test".to_string(),
                generation: 10,
            },
            top_attacks: vec![],
            config: RunConfig {
                mutation_rate: 0.3,
                crossover_rate: 0.7,
                tournament_size: 3,
                seed: Some(42),
            },
        };

        let json = serde_json::to_string(&results).unwrap();
        let parsed: EvolutionResults = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.target, "mock:comply");
        assert_eq!(parsed.generations, 10);
        assert_eq!(parsed.config.seed, Some(42));
    }

    #[test]
    fn test_run_config_serialization() {
        let config = RunConfig {
            mutation_rate: 0.3,
            crossover_rate: 0.7,
            tournament_size: 3,
            seed: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: RunConfig = serde_json::from_str(&json).unwrap();

        assert!((parsed.mutation_rate - 0.3).abs() < 1e-10);
        assert!(parsed.seed.is_none());
    }
}
