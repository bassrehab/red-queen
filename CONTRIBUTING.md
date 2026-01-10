# Contributing to Red Queen

Thank you for your interest in contributing to Red Queen!

## Code of Conduct

Be respectful, inclusive, and constructive. We're building tools for AI safety research.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/red-queen`
3. Create a branch: `git checkout -b feat/your-feature`
4. Make your changes
5. Run tests: `cargo test --workspace`
6. Submit a pull request

## Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/bassrehab/red-queen
cd red-queen
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

## Project Structure

```
red-queen/
├── crates/
│   ├── red-queen-core/     # Core evolution engine
│   ├── red-queen-llm/      # LLM safety domain
│   ├── red-queen-cli/      # Command-line interface
│   └── red-queen-python/   # Python bindings
├── docs/                   # Documentation
├── examples/               # Usage examples
└── tests/                  # Integration tests
```

## Pull Request Guidelines

### Before Submitting

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for significant changes)

### Commit Messages

```
<type>: <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code refactoring
- `test`: Adding tests
- `bench`: Performance improvements
- `chore`: Maintenance

Examples:
```
feat: add lexicase selection operator
fix: handle rate limit errors in OpenAI target
docs: add API examples for Python bindings
```

### Code Style

- Follow Rust conventions
- Document public APIs with rustdoc
- Write tests for new functionality
- Keep functions focused and small
- Use meaningful variable names

## Areas for Contribution

### Good First Issues

Look for issues labeled `good first issue`:
- Documentation improvements
- Adding examples
- Test coverage
- Small bug fixes

### Feature Contributions

Before starting major features, please open an issue to discuss:
- New selection operators
- New target adapters
- New domains (fraud, cyber, etc.)
- Performance improvements

### Research Contributions

We welcome research collaborations:
- Novel selection algorithms
- New behavior characterizations
- Experimental results
- Paper collaborations

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test

# Run benchmarks
cargo bench
```

## Documentation

- Use rustdoc for API documentation
- Update docs/ for user guides
- Include examples in doc comments

```rust
/// Performs tournament selection.
///
/// # Arguments
///
/// * `population` - The population to select from
/// * `tournament_size` - Number of individuals in each tournament
///
/// # Example
///
/// ```
/// let selection = Tournament::new(3);
/// let parent = selection.select(&population, &mut rng);
/// ```
pub fn select(&self, population: &Population<G>, rng: &mut impl Rng) -> &Individual<G>
```

## Questions?

- Open an issue for bugs or features
- Start a discussion for questions
- Email maintainers for security issues (see SECURITY.md)

Thank you for contributing!
