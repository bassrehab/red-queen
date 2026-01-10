//! Red Queen CLI

use clap::{Parser, Subcommand};

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
    /// Analyze results
    Analyze {
        /// Path to results file
        #[arg(short, long)]
        input: String,
    },
}

#[derive(Subcommand)]
enum LlmCommands {
    /// Discover jailbreaks
    Jailbreak {
        /// Target model (e.g., openai:gpt-4)
        #[arg(short, long)]
        target: String,

        /// Number of generations
        #[arg(short, long, default_value = "100")]
        generations: usize,

        /// Population size
        #[arg(short, long, default_value = "50")]
        population: usize,

        /// Output file
        #[arg(short, long, default_value = "results.json")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Llm { action } => match action {
            LlmCommands::Jailbreak {
                target,
                generations,
                population,
                output,
            } => {
                println!("Red Queen - LLM Jailbreak Discovery");
                println!("====================================");
                println!("Target: {}", target);
                println!("Generations: {}", generations);
                println!("Population: {}", population);
                println!("Output: {}", output);
                println!();
                println!("TODO: Implement evolution loop");

                // TODO: Implement actual evolution
                // 1. Parse target string (openai:gpt-4 -> OpenAITarget)
                // 2. Initialize population
                // 3. Run evolution
                // 4. Export results
            }
        },
        Commands::Analyze { input } => {
            println!("Analyzing results from: {}", input);
            // TODO: Implement analysis
        }
    }

    Ok(())
}
