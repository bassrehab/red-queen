# Architecture Overview

Red Queen is built as a modular Rust workspace with clear separation between the evolution engine and domain-specific implementations.

## Crate Structure

```
red-queen/
├── crates/
│   ├── red-queen-core/     # Core evolution engine
│   ├── red-queen-llm/      # LLM safety domain
│   ├── red-queen-cli/      # Command-line interface
│   └── red-queen-python/   # Python bindings (future)
├── docs/                   # Documentation (mdBook)
├── examples/               # Usage examples
└── tests/                  # Integration tests
```

## Dependency Graph

```
                    ┌─────────────────┐
                    │  red-queen-cli  │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
    ┌─────────────────┐  ┌─────────────────┐
    │ red-queen-core  │◄─│  red-queen-llm  │
    └─────────────────┘  └─────────────────┘
```

## Core Design Principles

### 1. Trait-Based Extensibility

All major components are defined as traits:

```rust
trait Genome: Clone + Send + Sync { ... }
trait Fitness<G: Genome>: Send + Sync { ... }
trait Selection<G: Genome>: Send + Sync { ... }
trait Judge: Send + Sync { ... }
```

This enables new domains without modifying core code.

### 2. Separation of Concerns

- **Core** knows nothing about LLMs
- **LLM domain** implements core traits
- **CLI** composes components

### 3. Parallel-Ready

All traits require `Send + Sync`, enabling:
- Parallel fitness evaluation
- Concurrent target queries
- Future island-model parallelism

### 4. Async for I/O

Network operations use async/await:

```rust
#[async_trait]
pub trait LLMTarget: Send + Sync {
    async fn query(&self, prompt: &str) -> Result<String, TargetError>;
}
```

## Data Flow

```
┌──────────┐     ┌──────────────┐     ┌─────────────┐
│ Genome   │────▶│ Fitness Eval │────▶│  Selection  │
│ (attack) │     │ (query LLM)  │     │ (tournament)│
└──────────┘     └──────────────┘     └──────┬──────┘
     ▲                                        │
     │           ┌──────────────┐             │
     └───────────│  Variation   │◄────────────┘
                 │ (mut/cross)  │
                 └──────────────┘
```

## Key Abstractions

### Evolution Engine

Orchestrates the evolution loop:

```rust
pub struct Evolution<G, F, S> {
    fitness: F,
    selection: S,
    population: Population<G>,
    config: EvolutionConfig,
}
```

### Population

Manages individuals:

```rust
pub struct Population<G: Genome> {
    individuals: Vec<Individual<G>>,
    config: PopulationConfig,
}
```

### Individual

Wraps a genome with metadata:

```rust
pub struct Individual<G: Genome> {
    pub genome: G,
    pub fitness: Option<FitnessValue>,
    pub behavior: Option<BehaviorDescriptor>,
    pub birth_generation: usize,
}
```

## Configuration

All configuration flows from CLI through typed structs:

```rust
pub struct EvolutionConfig {
    pub population: PopulationConfig,
    pub variation: VariationConfig,
    pub seed: Option<u64>,
}
```

## Error Handling

- Core uses `thiserror` for typed errors
- CLI uses `anyhow` for ergonomic error chains
- Async errors propagate through `Result`

## Further Reading

- [Core Components](./core-components.md) - Detailed core API
- [LLM Domain](./llm-domain.md) - LLM-specific implementation
- [Extensibility](./extensibility.md) - Adding new domains
