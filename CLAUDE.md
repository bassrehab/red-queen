# Red Queen - Claude Code Project Instructions

## Project Overview

Red Queen is an evolutionary adversarial testing framework. It uses quality-diversity evolution to discover diverse vulnerabilities in AI systems, starting with LLM safety.

**Core Philosophy**: The novel part isn't the genetic algorithm - it's the semantic-aware genome representations, multi-objective fitness functions, and quality-diversity approach to security testing.

---

## Key Documents

| Document | Purpose |
|----------|---------|
| `IDEATION.md` | Full concept, architecture, algorithms |
| `REPO_STRUCTURE.md` | Project structure, dependencies, CI/CD |
| `RESEARCH_NOVELTY.md` | Publication potential, research questions |
| `docs/ARCHITECTURE.md` | Technical architecture details |
| `docs/API_DESIGN.md` | API design principles |

---

## Technology Stack

- **Language**: Rust (core), Python bindings (PyO3)
- **Build**: Cargo workspace
- **Async**: Tokio
- **Parallelism**: Rayon
- **Serialization**: Serde
- **CLI**: Clap
- **Python**: PyO3 + Maturin

---

## Project Structure

```
red-queen/
├── crates/
│   ├── red-queen-core/     # Core evolution engine
│   ├── red-queen-llm/      # LLM safety domain
│   ├── red-queen-cli/      # Command-line interface
│   └── red-queen-python/   # Python bindings
├── python/                 # Python package wrapper
├── examples/               # Usage examples
├── docs/                   # Documentation
├── tests/                  # Integration tests
└── benches/                # Benchmarks
```

---

## Development Guidelines

### Code Style

- Follow Rust standard style (rustfmt)
- Use `clippy` for linting
- Document all public APIs with rustdoc
- Write tests for all core functionality

### Commit Messages

```
<type>: <description>

Types: feat, fix, docs, refactor, test, bench, chore
```

### Branch Strategy

- `main` - stable releases
- `dev` - development branch
- `feat/*` - feature branches
- `fix/*` - bug fixes

---

## Core Concepts

### 1. Genome

The representation of an individual (attack pattern). Red Queen uses **semantic genomes** that operate at meaning level, not token level.

```rust
// Example: LLM Attack Genome
struct LLMAttackGenome {
    strategy: Strategy,           // Roleplay, Encoding, Authority
    obfuscation: Vec<Encoding>,   // Base64, ROT13, etc.
    structure: PromptStructure,   // Prefix, core, suffix
}
```

### 2. Fitness

Evaluation function that scores how well an attack performs. Multi-objective fitness is common.

```rust
struct Fitness {
    success: f64,      // Did it bypass safety?
    transfer: f64,     // Works across models?
    stealth: f64,      // Is it detectable?
}
```

### 3. Selection

How parents are chosen for reproduction. Key algorithms:
- Tournament selection
- Lexicase selection (specialist preservation)
- Novelty-based selection

### 4. Variation

How offspring are created:
- Mutation (modify one individual)
- Crossover (combine two individuals)

### 5. Archive (Quality-Diversity)

MAP-Elites archive maintains diverse solutions across behavioral dimensions.

---

## Implementation Priorities

### Phase 1: Core Framework (Current)
1. Basic population management
2. Genome trait and encoding
3. Tournament selection
4. Standard mutation/crossover
5. Simple fitness evaluation

### Phase 2: LLM Domain
1. Semantic genome for LLM attacks
2. OpenAI/Anthropic target adapters
3. Jailbreak fitness function
4. CLI for experiments

### Phase 3: Quality-Diversity
1. MAP-Elites implementation
2. Novelty search
3. Behavior characterization

### Phase 4: Advanced Features
1. Co-evolution
2. Island model
3. Adaptive operators
4. Python bindings

---

## Testing Strategy

### Unit Tests
- All core algorithms in isolation
- Genome encoding/decoding
- Selection operators
- Variation operators

### Integration Tests
- Full evolution runs
- API adapter integration
- CLI commands

### Property-Based Tests
- Use `proptest` for invariant testing
- Population size consistency
- Fitness bounds

---

## API Design Principles

### Fluent Builder Pattern

```rust
let evolution = RedQueen::new()
    .with_population_size(100)
    .with_genome(LLMAttackGenome::default())
    .with_fitness(JailbreakFitness::new())
    .with_selection(Tournament::new(3))
    .build();
```

### Trait-Based Extensibility

```rust
trait Genome: Clone + Send + Sync {
    fn mutate(&mut self, rng: &mut impl Rng);
    fn crossover(&self, other: &Self) -> Self;
}

trait Fitness<G: Genome>: Send + Sync {
    fn evaluate(&self, genome: &G) -> FitnessValue;
}
```

### Error Handling

- Use `thiserror` for error types
- Return `Result<T, Error>` from fallible operations
- Provide context with `anyhow` in applications

---

## Security Considerations

1. **No credential storage** - API keys via environment only
2. **Rate limiting** - Built-in for all API targets
3. **Audit logging** - Log all attacks for accountability
4. **Responsible use** - See SECURITY.md

---

## Useful Commands

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Build Python bindings
cd crates/red-queen-python && maturin develop

# Run CLI
cargo run -p red-queen-cli -- --help
```

---

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `OPENAI_API_KEY` | OpenAI API access |
| `ANTHROPIC_API_KEY` | Anthropic API access |
| `GOOGLE_API_KEY` | Google AI API access |
| `RED_QUEEN_LOG` | Log level (debug, info, warn, error) |

---

## References

- `IDEATION.md` - Full project ideation
- `REPO_STRUCTURE.md` - Detailed structure
- `RESEARCH_NOVELTY.md` - Research directions
- [MAP-Elites Paper](https://arxiv.org/abs/1504.04909)
- [Novelty Search Paper](https://arxiv.org/abs/1106.5156)