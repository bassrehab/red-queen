# Testing

Guide to testing Red Queen.

## Running Tests

### All Tests

```bash
cargo test --workspace
```

### Specific Crate

```bash
cargo test -p red-queen-core
cargo test -p red-queen-llm
cargo test -p red-queen-cli
```

### Single Test

```bash
cargo test test_tournament_selection
```

### With Output

```bash
cargo test -- --nocapture
```

## Test Categories

### Unit Tests

Located in each module:

```rust
// src/selection.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tournament_new() {
        let tournament = Tournament::new(5);
        assert_eq!(tournament.size, 5);
    }
}
```

### Async Tests

For async functions (targets, judges):

```rust
#[tokio::test]
async fn test_mock_target_query() {
    let target = MockTarget::always_refuse();
    let response = target.query("test").await.unwrap();
    assert!(response.contains("I cannot"));
}
```

### Integration Tests

In `tests/` directory:

```rust
// tests/evolution_test.rs
#[test]
fn test_full_evolution_with_mock() {
    // Set up evolution with mock target
    // Run for N generations
    // Assert results
}
```

## Mock Targets

For testing without API calls:

```rust
// Always refuses
let target = MockTarget::always_refuse();

// Always complies
let target = MockTarget::always_comply();

// Custom responses
let target = MockTarget::with_responses(vec![
    "Response 1".to_string(),
    "Response 2".to_string(),
]);
```

## Test Coverage

Run with coverage:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html
```

View report in `tarpaulin-report.html`.

## CI Testing

Tests run automatically on:

- Pull requests
- Pushes to main

Required checks:

```yaml
- cargo test --workspace
- cargo clippy --workspace -- -D warnings
- cargo fmt --all -- --check
```

## Test Patterns

### Testing Genomes

```rust
#[test]
fn test_genome_mutation() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut genome = LLMAttackGenome::random(&mut rng);
    let original = genome.clone();

    genome.mutate(&mut rng, 1.0);  // 100% mutation rate

    // At least one field should change
    assert!(
        genome.primary_strategy != original.primary_strategy
        || genome.encoding != original.encoding
    );
}
```

### Testing Fitness

```rust
#[test]
fn test_fitness_with_refusal() {
    let target = Arc::new(MockTarget::always_refuse());
    let fitness = JailbreakFitness::new(target);
    let genome = make_test_genome();

    let result = fitness.evaluate(&genome);

    assert!(result.value.primary() < 0.5);  // Low score for refusal
}
```

### Testing Selection

```rust
#[test]
fn test_tournament_prefers_higher_fitness() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut pop = create_population_with_varied_fitness(&mut rng);

    let tournament = Tournament::new(5);
    let mut high_count = 0;

    for _ in 0..100 {
        let selected = tournament.select(&pop, &mut rng);
        if selected.fitness_value() > 0.8 {
            high_count += 1;
        }
    }

    // Should frequently select high-fitness individuals
    assert!(high_count > 50);
}
```

### Testing Judge

```rust
#[test]
fn test_heuristic_judge_refusal() {
    let judge = HeuristicJudge::new();

    let result = tokio_test::block_on(
        judge.evaluate("test", "I cannot help with that")
    ).unwrap();

    assert!(result.refused);
    assert!(result.score < 0.5);
}
```

## Debugging Tests

### Print Debug

```rust
#[test]
fn test_with_debug() {
    let genome = LLMAttackGenome::random(&mut rng);
    dbg!(&genome);  // Prints to stderr
    println!("Prompt: {}", genome.to_prompt());  // With --nocapture
}
```

### Test Isolation

Tests run in parallel by default. For isolation:

```rust
#[test]
#[serial]  // Requires serial_test crate
fn test_modifies_global_state() {
    // ...
}
```

## Performance Testing

```bash
cargo install cargo-criterion
cargo criterion
```

Benchmarks in `benches/`:

```rust
fn benchmark_fitness_evaluation(c: &mut Criterion) {
    let target = Arc::new(MockTarget::always_comply());
    let fitness = JailbreakFitness::new(target);
    let genome = make_test_genome();

    c.bench_function("fitness_evaluation", |b| {
        b.iter(|| fitness.evaluate(&genome))
    });
}
```
