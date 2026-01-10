# red-queen-core API

API reference for the core evolution engine.

## Modules

### `genome`

Core genome trait and behavior descriptors.

```rust
use red_queen_core::genome::{Genome, BehaviorDescriptor};
```

#### `Genome` trait

```rust
pub trait Genome: Clone + Send + Sync {
    type Phenotype;
    fn random<R: Rng>(rng: &mut R) -> Self;
    fn mutate<R: Rng>(&mut self, rng: &mut R, rate: f64);
    fn crossover<R: Rng>(&self, other: &Self, rng: &mut R) -> Self;
    fn to_phenotype(&self) -> Self::Phenotype;
}
```

#### `BehaviorDescriptor`

```rust
pub struct BehaviorDescriptor {
    pub values: Vec<f64>,
}

impl BehaviorDescriptor {
    pub fn new(values: Vec<f64>) -> Self;
    pub fn distance(&self, other: &Self) -> f64;
}
```

### `fitness`

Fitness evaluation traits and types.

```rust
use red_queen_core::fitness::{Fitness, FitnessValue, FitnessResult};
```

#### `Fitness` trait

```rust
pub trait Fitness<G: Genome>: Send + Sync {
    fn evaluate(&self, genome: &G) -> FitnessResult;
    fn evaluate_batch(&self, genomes: &[G]) -> Vec<FitnessResult>;
    fn maximizing(&self) -> bool;
}
```

#### `FitnessValue`

```rust
pub enum FitnessValue {
    Single(f64),
    Multi(Vec<f64>),
    Lexicase(Vec<(String, f64)>),
}

impl FitnessValue {
    pub fn primary(&self) -> f64;
    pub fn dominates(&self, other: &Self) -> bool;
}
```

### `population`

Population management.

```rust
use red_queen_core::population::{Population, Individual, PopulationConfig};
```

#### `Population<G>`

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

#### `Individual<G>`

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

### `selection`

Selection operators.

```rust
use red_queen_core::selection::{Selection, Tournament};
```

#### `Selection` trait

```rust
pub trait Selection<G: Genome>: Send + Sync {
    fn select<'a, R: Rng>(
        &self,
        population: &'a Population<G>,
        rng: &mut R,
    ) -> &'a Individual<G>;
}
```

#### `Tournament`

```rust
pub struct Tournament {
    pub size: usize,
}

impl Tournament {
    pub fn new(size: usize) -> Self;
}
```

### `evolution`

Evolution engine.

```rust
use red_queen_core::evolution::{Evolution, EvolutionConfig, EvolutionResult};
```

#### `Evolution<G, F, S>`

```rust
pub struct Evolution<G, F, S>
where
    G: Genome,
    F: Fitness<G>,
    S: Selection<G>,
{
    // ...
}

impl<G, F, S> Evolution<G, F, S> {
    pub fn new(fitness: F, selection: S, config: EvolutionConfig) -> Self;
    pub fn step(&mut self) -> &Population<G>;
    pub fn run(&mut self, generations: usize) -> EvolutionResult<G>;
    pub fn current_generation(&self) -> usize;
}
```

#### `EvolutionConfig`

```rust
pub struct EvolutionConfig {
    pub population: PopulationConfig,
    pub variation: VariationConfig,
    pub seed: Option<u64>,
}
```

### `archive`

Quality-diversity archive.

```rust
use red_queen_core::archive::Archive;
```

#### `Archive<G>`

```rust
pub struct Archive<G: Genome> {
    // ...
}

impl<G: Genome> Archive<G> {
    pub fn new(dimensions: Vec<ArchiveDimension>) -> Self;
    pub fn add(&mut self, individual: Individual<G>) -> bool;
    pub fn get(&self, behavior: &BehaviorDescriptor) -> Option<&Individual<G>>;
    pub fn solutions(&self) -> impl Iterator<Item = &Individual<G>>;
    pub fn coverage(&self) -> f64;
}
```

### `variation`

Variation operators configuration.

```rust
use red_queen_core::variation::VariationConfig;
```

#### `VariationConfig`

```rust
pub struct VariationConfig {
    pub mutation_rate: f64,
    pub crossover_rate: f64,
}
```

## Prelude

Common imports:

```rust
use red_queen_core::prelude::*;
// Imports: Genome, Fitness, Selection, Evolution, etc.
```
