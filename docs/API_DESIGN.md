# Red Queen API Design

## Design Principles

1. **Fluent Builder Pattern** - Configuration via method chaining
2. **Trait-Based Extensibility** - Core behaviors as traits
3. **Sensible Defaults** - Works out of the box
4. **Type Safety** - Leverage Rust's type system
5. **Async-First** - Native async for I/O operations

---

## Rust API

### Builder Pattern

```rust
use red_queen::prelude::*;
use red_queen_llm::prelude::*;

// Minimal configuration
let evolution = RedQueen::new()
    .with_genome(LLMAttackGenome::default())
    .with_fitness(JailbreakFitness::new(OpenAI::new("gpt-4")))
    .build()?;

// Full configuration
let evolution = RedQueen::new()
    // Population settings
    .with_population_size(100)
    .with_elitism(5)

    // Genome
    .with_genome(LLMAttackGenome::builder()
        .with_strategies(vec![
            AttackStrategy::Roleplay,
            AttackStrategy::Encoding,
        ])
        .with_topics(vec![
            HarmCategory::Violence,
            HarmCategory::Illegal,
        ])
        .build())

    // Fitness
    .with_fitness(JailbreakFitness::builder()
        .with_target(OpenAI::new("gpt-4"))
        .with_judge(LLMJudge::new("gpt-4"))
        .with_metrics(vec![
            Metric::Success,
            Metric::Stealth,
            Metric::Naturalness,
        ])
        .build())

    // Selection
    .with_selection(Lexicase::new())

    // Variation
    .with_mutation_rate(0.3)
    .with_crossover_rate(0.7)

    // Quality-Diversity
    .with_archive(MapElites::builder()
        .with_dimension("strategy", 0.0, 5.0, 5)
        .with_dimension("obfuscation", 0.0, 3.0, 3)
        .with_dimension("length", 0.0, 1000.0, 10)
        .build())

    // Parallelism
    .with_threads(8)
    .with_batch_size(20)

    // Logging
    .with_logger(FileLogger::new("experiment.log"))
    .with_checkpoint_interval(50)

    .build()?;
```

### Running Evolution

```rust
// Synchronous run
let results = evolution.run(generations: 500)?;

// Async run with progress
let results = evolution.run_async(500, |progress| {
    println!("Generation {}: best = {:.3}",
        progress.generation,
        progress.best_fitness
    );
}).await?;

// Run until condition
let results = evolution.run_until(|state| {
    state.best_fitness > 0.95 || state.generation > 1000
})?;
```

### Accessing Results

```rust
// Best individual
let best = results.best();
println!("Best attack: {}", best.genome.to_phenotype());
println!("Fitness: {:?}", best.fitness);

// Archive (QD)
for individual in results.archive.iter() {
    println!("{}: {:.3}", individual.behavior, individual.fitness);
}

// Coverage statistics
let coverage = results.archive.coverage();
println!("Cells filled: {}/{}", coverage.filled, coverage.total);
println!("Coverage: {:.1}%", coverage.percentage * 100.0);

// Export
results.export_json("results.json")?;
results.export_csv("results.csv")?;
results.archive.visualize("archive.png")?;
```

---

## Python API

### Installation

```bash
pip install red-queen
```

### Basic Usage

```python
from red_queen import RedQueen
from red_queen.llm import (
    LLMAttackGenome,
    JailbreakFitness,
    AttackStrategy,
    HarmCategory,
)
from red_queen.targets import OpenAI, Anthropic
from red_queen.selection import Lexicase, Tournament
from red_queen.archives import MapElites

# Simple configuration
evolution = (
    RedQueen()
    .with_genome(LLMAttackGenome())
    .with_fitness(JailbreakFitness(target=OpenAI("gpt-4")))
    .build()
)

results = evolution.run(generations=500)
```

### Advanced Configuration

```python
# Genome configuration
genome = LLMAttackGenome(
    strategies=[
        AttackStrategy.ROLEPLAY,
        AttackStrategy.ENCODING,
        AttackStrategy.HYPOTHETICAL,
    ],
    topics=[
        HarmCategory.VIOLENCE,
        HarmCategory.ILLEGAL,
    ],
    max_length=1000,
)

# Multi-target fitness
fitness = JailbreakFitness(
    targets=[
        OpenAI("gpt-4"),
        Anthropic("claude-3-opus"),
    ],
    judge=OpenAI("gpt-4"),
    metrics=["success", "transfer", "stealth"],
)

# MAP-Elites archive
archive = MapElites(
    dimensions={
        "strategy": (0, 5, 5),      # (min, max, bins)
        "obfuscation": (0, 3, 3),
        "length": (0, 1000, 10),
    }
)

# Full configuration
evolution = (
    RedQueen()
    .with_population_size(100)
    .with_genome(genome)
    .with_fitness(fitness)
    .with_selection(Lexicase())
    .with_mutation_rate(0.3)
    .with_crossover_rate(0.7)
    .with_archive(archive)
    .with_threads(8)
    .build()
)
```

### Progress Callbacks

```python
def on_progress(progress):
    print(f"Gen {progress.generation}: "
          f"best={progress.best_fitness:.3f}, "
          f"coverage={progress.coverage:.1%}")

results = evolution.run(
    generations=500,
    callback=on_progress,
    callback_interval=10,
)
```

### Async Support

```python
import asyncio

async def run_experiment():
    evolution = RedQueen().with_genome(...).with_fitness(...).build()

    results = await evolution.run_async(generations=500)
    return results

results = asyncio.run(run_experiment())
```

### Working with Results

```python
# Best individual
best = results.best()
print(f"Best attack:\n{best.genome.to_prompt()}")
print(f"Fitness: {best.fitness}")

# Iterate archive
for attack in results.archive:
    print(f"Strategy: {attack.genome.strategy}")
    print(f"Success: {attack.fitness.success:.2%}")
    print(f"Behavior: {attack.behavior}")
    print()

# Filter archive
roleplay_attacks = [
    a for a in results.archive
    if a.genome.strategy == AttackStrategy.ROLEPLAY
]

# Export
results.to_json("results.json")
results.to_csv("results.csv")
results.to_dataframe()  # Returns pandas DataFrame

# Visualization
results.archive.plot_heatmap("coverage.png")
results.plot_fitness_over_time("fitness.png")
```

### Custom Genome

```python
from red_queen import Genome
import random

class CustomGenome(Genome):
    def __init__(self, data: dict):
        self.data = data

    @classmethod
    def random(cls) -> "CustomGenome":
        return cls(data={
            "field1": random.random(),
            "field2": random.choice(["a", "b", "c"]),
        })

    def mutate(self, rate: float) -> None:
        if random.random() < rate:
            self.data["field1"] += random.gauss(0, 0.1)

    def crossover(self, other: "CustomGenome") -> "CustomGenome":
        return CustomGenome(data={
            "field1": (self.data["field1"] + other.data["field1"]) / 2,
            "field2": random.choice([self.data["field2"], other.data["field2"]]),
        })

    def to_phenotype(self) -> str:
        return f"Input: {self.data}"
```

### Custom Fitness

```python
from red_queen import Fitness, FitnessResult

class CustomFitness(Fitness):
    def __init__(self, target_system):
        self.target = target_system

    def evaluate(self, genome: CustomGenome) -> FitnessResult:
        # Convert to input
        input_data = genome.to_phenotype()

        # Test against target
        response = self.target.process(input_data)

        # Compute fitness
        success = self.is_successful(response)
        stealth = self.compute_stealth(input_data)

        return FitnessResult(
            values={"success": float(success), "stealth": stealth},
            behavior=self.characterize(genome),
        )
```

---

## CLI API

### Basic Commands

```bash
# Discover jailbreaks
red-queen llm jailbreak \
    --target openai:gpt-4 \
    --generations 500 \
    --population 100 \
    --output results.json

# Multi-target
red-queen llm jailbreak \
    --target openai:gpt-4 \
    --target anthropic:claude-3-opus \
    --target google:gemini-pro \
    --generations 500 \
    --output multi_target.json

# With specific strategies
red-queen llm jailbreak \
    --target openai:gpt-4 \
    --strategies roleplay,encoding,hypothetical \
    --topics violence,illegal \
    --generations 500

# Resume from checkpoint
red-queen llm jailbreak \
    --resume checkpoint_500.json \
    --generations 1000
```

### Configuration File

```yaml
# config.yaml
population:
  size: 100
  elitism: 5

genome:
  type: llm-attack
  strategies:
    - roleplay
    - encoding
    - hypothetical
  topics:
    - violence
    - illegal

fitness:
  targets:
    - type: openai
      model: gpt-4
    - type: anthropic
      model: claude-3-opus
  judge:
    type: openai
    model: gpt-4

selection:
  type: lexicase

variation:
  mutation_rate: 0.3
  crossover_rate: 0.7

archive:
  type: map-elites
  dimensions:
    strategy: [0, 5, 5]
    obfuscation: [0, 3, 3]
    length: [0, 1000, 10]

output:
  directory: ./results
  checkpoints: true
  checkpoint_interval: 50
```

```bash
red-queen run --config config.yaml
```

### Analysis Commands

```bash
# Analyze results
red-queen analyze results.json \
    --coverage \
    --top-attacks 10 \
    --export-report report.html

# Visualize archive
red-queen visualize results.json \
    --type heatmap \
    --dimensions strategy,obfuscation \
    --output coverage.png

# Compare experiments
red-queen compare exp1.json exp2.json \
    --metric coverage \
    --output comparison.png
```

---

## Error Handling

### Rust

```rust
use red_queen::error::{Error, Result};

fn main() -> Result<()> {
    let evolution = RedQueen::new()
        .with_genome(LLMAttackGenome::default())
        .with_fitness(JailbreakFitness::new(OpenAI::new("gpt-4")))
        .build()?;  // Returns Error if misconfigured

    let results = evolution.run(500)?;  // Returns Error if evolution fails

    Ok(())
}

// Error types
pub enum Error {
    ConfigError(String),
    TargetError(String),
    EvaluationError(String),
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
}
```

### Python

```python
from red_queen.exceptions import (
    ConfigurationError,
    TargetError,
    EvaluationError,
    RateLimitError,
)

try:
    evolution = RedQueen().with_genome(...).build()
    results = evolution.run(500)
except ConfigurationError as e:
    print(f"Configuration error: {e}")
except TargetError as e:
    print(f"Target API error: {e}")
except RateLimitError as e:
    print(f"Rate limit exceeded: {e}")
    # Handle gracefully - maybe wait and retry
```

---

## Logging and Telemetry

```rust
// Enable tracing
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_env_filter("red_queen=debug")
    .init();

// Custom logger
let evolution = RedQueen::new()
    .with_logger(CompositeLogger::new(vec![
        Box::new(FileLogger::new("experiment.log")),
        Box::new(TensorBoardLogger::new("runs/exp1")),
        Box::new(WandbLogger::new("project", "experiment")),
    ]))
    .build()?;
```

```python
import logging

logging.basicConfig(level=logging.DEBUG)

# Or use structured logging
evolution = (
    RedQueen()
    .with_logger("tensorboard", logdir="runs/exp1")
    .with_logger("wandb", project="red-queen", name="exp1")
    .build()
)
```

---

## Rate Limiting

```rust
let target = OpenAI::builder()
    .with_model("gpt-4")
    .with_rate_limit(RateLimit {
        requests_per_minute: 60,
        tokens_per_minute: 10000,
    })
    .with_retry(RetryConfig {
        max_retries: 3,
        backoff: ExponentialBackoff::default(),
    })
    .build()?;
```

```python
target = OpenAI(
    model="gpt-4",
    rate_limit=RateLimit(
        requests_per_minute=60,
        tokens_per_minute=10000,
    ),
    retry=RetryConfig(
        max_retries=3,
        backoff="exponential",
    ),
)
```
