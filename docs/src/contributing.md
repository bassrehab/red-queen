# Contributing

Thank you for your interest in contributing to Red Queen!

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch

```bash
git clone https://github.com/YOUR_USERNAME/red-queen.git
cd red-queen
git checkout -b feat/my-feature
```

## Development Setup

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

## Code Style

### Rust Guidelines

- Follow `rustfmt` defaults
- Use `clippy` with warnings as errors
- Document all public APIs with rustdoc
- Write tests for new functionality

### Commit Messages

Use conventional commits:

```
<type>: <description>

Types:
- feat: New feature
- fix: Bug fix
- docs: Documentation
- refactor: Code refactoring
- test: Adding tests
- bench: Benchmarks
- chore: Maintenance
```

Examples:

```
feat: add Gemini target adapter
fix: handle rate limiting in OpenAI target
docs: update CLI reference
```

## Pull Requests

### Before Submitting

1. Run the full test suite
2. Ensure clippy passes
3. Format code with rustfmt
4. Update documentation if needed

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
```

### PR Guidelines

- Keep PRs focused and small
- Write clear descriptions
- Reference related issues
- Add tests for new code

## Project Structure

```
crates/
├── red-queen-core/     # Core evolution - generic algorithms
├── red-queen-llm/      # LLM domain - attack genomes, targets
├── red-queen-cli/      # CLI - user interface
└── red-queen-python/   # Python bindings (future)
```

### Where to Contribute

| Area | Crate | Examples |
|------|-------|----------|
| Evolution algorithms | core | Selection methods, archive |
| LLM targets | llm | New providers, judges |
| Attack strategies | llm | New strategies, encodings |
| User interface | cli | Commands, output formats |
| Documentation | docs | Guides, API docs |

## Testing

### Unit Tests

Each module should have tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // ...
    }
}
```

### Integration Tests

For cross-crate functionality:

```rust
// tests/integration_test.rs
#[test]
fn test_full_evolution_run() {
    // ...
}
```

### Property-Based Tests

For invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn genome_crossover_valid(a: TestGenome, b: TestGenome) {
        let offspring = a.crossover(&b, &mut rng);
        // Assert invariants
    }
}
```

## Adding Features

### New Target

1. Implement `LLMTarget` in `red-queen-llm/src/targets.rs`
2. Add tests
3. Register in CLI `parse_target`
4. Update documentation

### New Attack Strategy

1. Add variant to `AttackStrategy` enum
2. Implement `format_prompt` case
3. Add behavior value
4. Update mutation logic
5. Add tests

### New Selection Method

1. Implement `Selection` trait
2. Add to CLI options
3. Write tests
4. Document

## Security

### Responsible Disclosure

If you discover a security vulnerability:

1. Do NOT open a public issue
2. Email security concerns privately
3. Allow time for a fix before disclosure

### Code Review

All PRs require review for:

- Security implications
- API design
- Test coverage
- Documentation

## Questions?

- Open an issue for discussion
- Tag with `question` label

Thank you for contributing!
