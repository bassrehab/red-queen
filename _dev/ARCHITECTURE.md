# Red Queen Architecture

## System Overview

Red Queen is a modular evolutionary computation framework designed for adversarial testing. The architecture separates core evolution mechanics from domain-specific implementations.

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Application Layer                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │     CLI     │  │   Python    │  │    WASM     │                  │
│  │             │  │   Bindings  │  │   (future)  │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
└────────────────────────────┬────────────────────────────────────────┘
                             │
┌────────────────────────────┴───────────────────────────────────────┐
│                           Domain Layer                             │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                      LLM Safety Domain                      │   │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐ │   │
│  │  │  Semantic │  │ Jailbreak │  │   Target  │  │  Behavior │ │   │
│  │  │   Genome  │  │  Fitness  │  │  Adapters │  │  Metrics  │ │   │
│  │  └───────────┘  └───────────┘  └───────────┘  └───────────┘ │   │
│  └─────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Future Domains                           │   │
│  │     Fraud Detection  │  Cybersecurity  │  Custom            │   │
│  └─────────────────────────────────────────────────────────────┘   │
└────────────────────────────┬───────────────────────────────────────┘
                             │
┌────────────────────────────┴───────────────────────────────────────┐
│                            Core Layer                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │
│  │ Population  │  │  Selection  │  │  Variation  │  │  Archives │  │
│  │ Management  │  │  Operators  │  │  Operators  │  │ (QD)      │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │
│  │   Genome    │  │   Fitness   │  │   Island    │  │  Logging  │  │
│  │   Traits    │  │   Traits    │  │   Model     │  │ Telemetry │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Population Management

Manages the lifecycle of individuals in the evolutionary process.

```rust
pub struct Population<G: Genome> {
    individuals: Vec<Individual<G>>,
    generation: usize,
    config: PopulationConfig,
}

pub struct Individual<G: Genome> {
    pub genome: G,
    pub fitness: Option<FitnessValue>,
    pub behavior: Option<BehaviorDescriptor>,
    pub metadata: IndividualMetadata,
}

pub struct PopulationConfig {
    pub size: usize,
    pub elitism: usize,           // Keep top N unchanged
    pub offspring_count: usize,   // New individuals per generation
}
```

**Responsibilities:**
- Initialize population (random or seeded)
- Track generation count
- Manage individual metadata (age, lineage)
- Support parallel evaluation

### 2. Genome Trait

Abstract interface for evolvable representations.

```rust
pub trait Genome: Clone + Send + Sync + Sized {
    /// Create a random genome
    fn random(rng: &mut impl Rng) -> Self;

    /// Mutate the genome in place
    fn mutate(&mut self, rng: &mut impl Rng, rate: f64);

    /// Combine two genomes to produce offspring
    fn crossover(&self, other: &Self, rng: &mut impl Rng) -> Self;

    /// Serialize to phenotype (the actual attack/input)
    fn to_phenotype(&self) -> Phenotype;

    /// Optional: distance metric for novelty search
    fn distance(&self, other: &Self) -> f64 {
        0.0 // Default: no distance defined
    }
}
```

**Key Insight**: Genomes are abstract representations. The `to_phenotype()` method converts them to concrete inputs (e.g., prompt strings).

### 3. Fitness Trait

Abstract interface for evaluation functions.

```rust
pub trait Fitness<G: Genome>: Send + Sync {
    /// Evaluate a single genome
    fn evaluate(&self, genome: &G) -> FitnessResult;

    /// Batch evaluation for efficiency
    fn evaluate_batch(&self, genomes: &[G]) -> Vec<FitnessResult> {
        genomes.iter().map(|g| self.evaluate(g)).collect()
    }

    /// Whether higher fitness is better
    fn maximizing(&self) -> bool {
        true
    }
}

pub struct FitnessResult {
    pub value: FitnessValue,
    pub behavior: Option<BehaviorDescriptor>,
    pub metadata: EvaluationMetadata,
}

pub enum FitnessValue {
    Single(f64),
    Multi(Vec<f64>),              // For multi-objective
    Lexicase(Vec<(String, f64)>), // For lexicase selection
}
```

### 4. Selection Operators

Choose parents for reproduction.

```rust
pub trait Selection<G: Genome>: Send + Sync {
    /// Select one parent from the population
    fn select<'a>(
        &self,
        population: &'a Population<G>,
        rng: &mut impl Rng,
    ) -> &'a Individual<G>;

    /// Select multiple parents
    fn select_many<'a>(
        &self,
        population: &'a Population<G>,
        count: usize,
        rng: &mut impl Rng,
    ) -> Vec<&'a Individual<G>> {
        (0..count).map(|_| self.select(population, rng)).collect()
    }
}
```

**Implementations:**

| Operator | Description | When to Use |
|----------|-------------|-------------|
| `Tournament` | Best of K random | General purpose |
| `Lexicase` | Sequential case filtering | Diverse specialists |
| `Novelty` | Based on behavioral novelty | Exploration |
| `NSGA2` | Pareto dominance | Multi-objective |
| `Epsilon` | Epsilon-greedy exploration | Balancing explore/exploit |

### 5. Variation Operators

Create offspring from parents.

```rust
pub trait Mutation<G: Genome>: Send + Sync {
    fn mutate(&self, genome: &mut G, rng: &mut impl Rng);
}

pub trait Crossover<G: Genome>: Send + Sync {
    fn crossover(&self, parent1: &G, parent2: &G, rng: &mut impl Rng) -> G;
}

pub struct VariationConfig {
    pub mutation_rate: f64,    // Probability of mutation
    pub crossover_rate: f64,   // Probability of crossover
    pub mutation_strength: f64, // Magnitude of mutations
}
```

### 6. Archives (Quality-Diversity)

Maintain diverse solutions across behavioral dimensions.

```rust
pub trait Archive<G: Genome>: Send + Sync {
    /// Try to add an individual to the archive
    fn add(&mut self, individual: Individual<G>) -> bool;

    /// Get all archived individuals
    fn get_all(&self) -> Vec<&Individual<G>>;

    /// Get coverage statistics
    fn coverage(&self) -> ArchiveCoverage;

    /// Sample from archive for reproduction
    fn sample(&self, rng: &mut impl Rng) -> Option<&Individual<G>>;
}

pub struct MapElitesArchive<G: Genome> {
    grid: HashMap<BehaviorCell, Individual<G>>,
    dimensions: Vec<BehaviorDimension>,
    resolution: Vec<usize>,
}

pub struct BehaviorDimension {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub bins: usize,
}
```

**MAP-Elites Key Concept**: The archive is a grid where each cell represents a region of behavior space. Only the best individual for each cell is kept, ensuring diversity.

---

## Evolution Loop

```rust
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
}

impl<G, F, S> Evolution<G, F, S> {
    pub fn run(&mut self, generations: usize) -> EvolutionResult<G> {
        // Initialize
        self.evaluate_population();

        for gen in 0..generations {
            // Selection
            let parents = self.select_parents();

            // Variation
            let offspring = self.create_offspring(&parents);

            // Evaluation
            self.evaluate_batch(&offspring);

            // Archive update (if QD)
            if let Some(archive) = &mut self.archive {
                for ind in &offspring {
                    archive.add(ind.clone());
                }
            }

            // Survivor selection
            self.population = self.select_survivors(offspring);

            // Logging
            self.log_generation(gen);
        }

        self.build_result()
    }
}
```

---

## LLM Safety Domain

### Semantic Genome

```rust
pub struct LLMAttackGenome {
    // Strategy layer
    pub primary_strategy: AttackStrategy,
    pub secondary_strategy: Option<AttackStrategy>,

    // Framing layer
    pub persona: Option<Persona>,
    pub scenario: Option<Scenario>,

    // Obfuscation layer
    pub encodings: Vec<Encoding>,
    pub language_mixing: Option<LanguageMixing>,

    // Structure layer
    pub prefix: PromptSection,
    pub core: PromptSection,
    pub suffix: PromptSection,

    // Target layer
    pub target_behavior: TargetBehavior,
    pub topic: HarmCategory,
}

pub enum AttackStrategy {
    Roleplay { persona: String, scenario: String },
    Encoding { method: EncodingMethod },
    Authority { claim: AuthorityClaim },
    Hypothetical { framing: String },
    MultiTurn { setup_turns: Vec<String> },
    Jailbreak { template: JailbreakTemplate },
}

pub enum Encoding {
    Base64,
    ROT13,
    Leetspeak,
    PigLatin,
    Unicode,
    Custom(String),
}
```

### Jailbreak Fitness

```rust
pub struct JailbreakFitness {
    target: Box<dyn LLMTarget>,
    judge: Box<dyn SafetyJudge>,
    config: FitnessConfig,
}

impl Fitness<LLMAttackGenome> for JailbreakFitness {
    fn evaluate(&self, genome: &LLMAttackGenome) -> FitnessResult {
        // Convert genome to prompt
        let prompt = genome.to_phenotype();

        // Query target LLM
        let response = self.target.query(&prompt);

        // Judge if jailbreak succeeded
        let success = self.judge.is_jailbroken(&prompt, &response);

        // Compute additional metrics
        let stealth = self.compute_stealth(&prompt);
        let naturalness = self.compute_naturalness(&prompt);

        // Compute behavior for MAP-Elites
        let behavior = self.characterize_behavior(genome);

        FitnessResult {
            value: FitnessValue::Multi(vec![
                success as i32 as f64,
                stealth,
                naturalness,
            ]),
            behavior: Some(behavior),
            metadata: EvaluationMetadata::default(),
        }
    }
}
```

### Target Adapters

```rust
#[async_trait]
pub trait LLMTarget: Send + Sync {
    async fn query(&self, prompt: &str) -> LLMResponse;
    fn name(&self) -> &str;
    fn rate_limit(&self) -> RateLimit;
}

pub struct OpenAITarget {
    client: OpenAIClient,
    model: String,
    config: TargetConfig,
}

pub struct AnthropicTarget {
    client: AnthropicClient,
    model: String,
    config: TargetConfig,
}

pub struct LocalTarget {
    endpoint: String,
    config: TargetConfig,
}
```

---

## Concurrency Model

### Parallel Evaluation

```rust
impl<G, F, S> Evolution<G, F, S> {
    fn evaluate_batch(&self, individuals: &mut [Individual<G>]) {
        // Use rayon for CPU parallelism
        individuals.par_iter_mut().for_each(|ind| {
            ind.fitness = Some(self.fitness.evaluate(&ind.genome));
        });
    }
}
```

### Async API Calls

```rust
impl OpenAITarget {
    async fn query_batch(&self, prompts: &[String]) -> Vec<LLMResponse> {
        // Concurrent API calls with rate limiting
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent));

        let futures: Vec<_> = prompts
            .iter()
            .map(|prompt| {
                let sem = semaphore.clone();
                let client = self.client.clone();
                async move {
                    let _permit = sem.acquire().await;
                    client.complete(prompt).await
                }
            })
            .collect();

        futures::future::join_all(futures).await
    }
}
```

### Island Model

```rust
pub struct IslandModel<G: Genome> {
    islands: Vec<Population<G>>,
    migration_interval: usize,
    migration_count: usize,
    topology: MigrationTopology,
}

impl<G: Genome> IslandModel<G> {
    fn migrate(&mut self) {
        match self.topology {
            MigrationTopology::Ring => self.migrate_ring(),
            MigrationTopology::FullyConnected => self.migrate_full(),
            MigrationTopology::Random => self.migrate_random(),
        }
    }

    fn migrate_ring(&mut self) {
        let n = self.islands.len();
        for i in 0..n {
            let emigrants = self.islands[i].get_best(self.migration_count);
            self.islands[(i + 1) % n].add_immigrants(emigrants);
        }
    }
}
```

---

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      Evolution Loop                             │
│                                                                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐ │
│  │Initialize│───►│Selection │───►│Variation │───►│Evaluation│ │
│  │Population│    │          │    │          │    │          │ │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘ │
│       ▲                                               │        │
│       │              ┌──────────┐                     │        │
│       │              │ Archive  │◄────────────────────┤        │
│       │              │ (QD)     │                     │        │
│       │              └──────────┘                     │        │
│       │                                               │        │
│       └───────────────────────────────────────────────┘        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Results Export  │
                    │  - JSON          │
                    │  - Visualization │
                    │  - Reports       │
                    └──────────────────┘
```

---

## Extension Points

### Adding a New Domain

1. Define a `Genome` type for your domain
2. Implement `Genome` trait
3. Define fitness function implementing `Fitness` trait
4. Define behavior dimensions for QD
5. Register in domain registry

### Adding a New Selection Operator

1. Implement `Selection` trait
2. Add to selection factory
3. Register in CLI

### Adding a New Target

1. Implement `LLMTarget` trait (or domain-specific target trait)
2. Handle authentication
3. Implement rate limiting
4. Add to target factory

---

## Performance Considerations

| Component | Strategy |
|-----------|----------|
| Population | Pre-allocate, reuse buffers |
| Fitness eval | Batch API calls, parallel |
| Archive lookup | Hash-based cell lookup |
| Serialization | Zero-copy where possible |
| Memory | Stream results, don't hold all |

---

## Future Considerations

1. **Distributed Evolution** - Multi-node island model
2. **GPU Acceleration** - For local model evaluation
3. **Persistent Archive** - Resume experiments
4. **Real-time Visualization** - Web dashboard
5. **Automated Reporting** - Generate vulnerability reports
