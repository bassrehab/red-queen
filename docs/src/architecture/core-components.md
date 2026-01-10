# Core Components

Detailed documentation of the `red-queen-core` crate.

## Genome Trait

The fundamental abstraction for evolved solutions:

```rust
pub trait Genome: Clone + Send + Sync {
    type Phenotype;

    /// Create a random genome
    fn random<R: Rng>(rng: &mut R) -> Self;

    /// Mutate in place
    fn mutate<R: Rng>(&mut self, rng: &mut R, rate: f64);

    /// Create offspring from two parents
    fn crossover<R: Rng>(&self, other: &Self, rng: &mut R) -> Self;

    /// Convert to phenotype (the expressed form)
    fn to_phenotype(&self) -> Self::Phenotype;
}
```

### Behavior Descriptor

Optional behavior characterization for QD:

```rust
pub struct BehaviorDescriptor {
    pub values: Vec<f64>,
}

impl BehaviorDescriptor {
    pub fn distance(&self, other: &Self) -> f64;
}
```

## Fitness Trait

Evaluates genomes:

```rust
pub trait Fitness<G: Genome>: Send + Sync {
    /// Evaluate a single genome
    fn evaluate(&self, genome: &G) -> FitnessResult;

    /// Batch evaluation (default: sequential)
    fn evaluate_batch(&self, genomes: &[G]) -> Vec<FitnessResult>;

    /// Higher is better (default: true)
    fn maximizing(&self) -> bool;
}
```

### Fitness Value

Supports multiple fitness paradigms:

```rust
pub enum FitnessValue {
    /// Single objective
    Single(f64),
    /// Multi-objective (Pareto)
    Multi(Vec<f64>),
    /// Per-case for lexicase selection
    Lexicase(Vec<(String, f64)>),
}
```

### Fitness Result

Complete evaluation output:

```rust
pub struct FitnessResult {
    pub value: FitnessValue,
    pub behavior: Option<BehaviorDescriptor>,
    pub metadata: EvaluationMetadata,
}
```

## Selection Trait

Chooses parents for reproduction:

```rust
pub trait Selection<G: Genome>: Send + Sync {
    fn select<'a, R: Rng>(
        &self,
        population: &'a Population<G>,
        rng: &mut R,
    ) -> &'a Individual<G>;
}
```

### Tournament Selection

The default implementation:

```rust
pub struct Tournament {
    pub size: usize,
}
```

## Population

Manages the collection of individuals:

```rust
pub struct Population<G: Genome> {
    pub individuals: Vec<Individual<G>>,
    pub config: PopulationConfig,
}

impl<G: Genome> Population<G> {
    pub fn random<R: Rng>(config: PopulationConfig, rng: &mut R) -> Self;
    pub fn top_n(&self, n: usize) -> Vec<&Individual<G>>;
    pub fn best(&self) -> Option<&Individual<G>>;
}
```

### Individual

Wraps genome with fitness and metadata:

```rust
pub struct Individual<G: Genome> {
    pub genome: G,
    pub fitness: Option<FitnessValue>,
    pub behavior: Option<BehaviorDescriptor>,
    pub birth_generation: usize,
}

impl<G: Genome> Individual<G> {
    pub fn fitness_value(&self) -> f64;
}
```

## Evolution Engine

Orchestrates the evolution process:

```rust
pub struct Evolution<G, F, S>
where
    G: Genome,
    F: Fitness<G>,
    S: Selection<G>,
{
    fitness: F,
    selection: S,
    population: Population<G>,
    config: EvolutionConfig,
    generation: usize,
}

impl<G, F, S> Evolution<G, F, S> {
    pub fn new(fitness: F, selection: S, config: EvolutionConfig) -> Self;
    pub fn step(&mut self) -> &Population<G>;
    pub fn run(&mut self, generations: usize) -> EvolutionResult<G>;
}
```

### Evolution Result

```rust
pub struct EvolutionResult<G: Genome> {
    pub best: Individual<G>,
    pub population: Population<G>,
    pub generations: usize,
}
```

## Archive (QD)

Stores diverse solutions:

```rust
pub struct Archive<G: Genome> {
    solutions: HashMap<GridCell, Individual<G>>,
    dimensions: Vec<ArchiveDimension>,
}

impl<G: Genome> Archive<G> {
    pub fn add(&mut self, individual: Individual<G>) -> bool;
    pub fn get(&self, behavior: &BehaviorDescriptor) -> Option<&Individual<G>>;
    pub fn coverage(&self) -> f64;
}
```

## Variation

Mutation and crossover configuration:

```rust
pub struct VariationConfig {
    pub mutation_rate: f64,
    pub crossover_rate: f64,
}
```

## RNG

Uses `rand` with optional seeding:

```rust
// Seeded for reproducibility
let rng = ChaCha8Rng::seed_from_u64(42);

// Or random
let rng = rand::thread_rng();
```

## Thread Safety

All traits require `Send + Sync`, enabling safe parallel evolution.
