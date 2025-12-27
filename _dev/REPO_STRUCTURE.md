# Red Queen - Repository Structure

## Suggested GitHub Repo Name

**`red-queen`** or **`redqueen`**

Alternative available names to check:
- `red-queen-evolution`
- `red-queen-ai`
- `adversarial-evolution`

## Initial Repo Structure

```
red-queen/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml              # Build, test, lint
│   │   ├── release.yml         # Automated releases
│   │   └── security.yml        # Security scanning
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   ├── feature_request.md
│   │   └── domain_request.md   # New domain support
│   └── PULL_REQUEST_TEMPLATE.md
│
├── crates/
│   ├── red-queen-core/         # Core evolution engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── population.rs
│   │       ├── genome.rs
│   │       ├── selection/
│   │       ├── variation/
│   │       └── archives/
│   │
│   ├── red-queen-llm/          # LLM safety domain
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── genome.rs       # LLM attack genome
│   │       ├── fitness.rs      # Jailbreak fitness
│   │       ├── mutations.rs    # Semantic mutations
│   │       └── targets/        # API adapters
│   │
│   ├── red-queen-cli/          # Command-line interface
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   └── red-queen-python/       # Python bindings (PyO3)
│       ├── Cargo.toml
│       ├── pyproject.toml
│       └── src/
│           └── lib.rs
│
├── python/
│   └── red_queen/              # Python package wrapper
│       ├── __init__.py
│       └── examples/
│
├── examples/
│   ├── jailbreak_discovery.rs
│   ├── prompt_injection.rs
│   └── multi_model_transfer.rs
│
├── notebooks/
│   ├── getting_started.ipynb
│   ├── analyzing_results.ipynb
│   └── custom_fitness.ipynb
│
├── docs/
│   ├── book/                   # mdBook documentation
│   │   ├── src/
│   │   │   ├── SUMMARY.md
│   │   │   ├── introduction.md
│   │   │   ├── concepts/
│   │   │   ├── tutorials/
│   │   │   └── api/
│   │   └── book.toml
│   └── research/               # Research notes, papers
│       └── references.md
│
├── tests/
│   ├── integration/
│   └── fixtures/
│
├── benches/                    # Performance benchmarks
│   └── evolution_bench.rs
│
├── Cargo.toml                  # Workspace manifest
├── Cargo.lock
├── LICENSE                     # Apache 2.0 or MIT
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md                 # Responsible disclosure policy
├── CHANGELOG.md
└── rustfmt.toml
```

## Cargo Workspace Setup

```toml
# Cargo.toml (root)
[workspace]
resolver = "2"
members = [
    "crates/red-queen-core",
    "crates/red-queen-llm",
    "crates/red-queen-cli",
    "crates/red-queen-python",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
repository = "https://github.com/bassrehab/red-queen"
authors = ["Subhadip Mitra"]

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rayon = "1.8"
rand = "0.8"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `rayon` | Parallel evaluation |
| `tokio` | Async API calls |
| `reqwest` | HTTP client for LLM APIs |
| `serde` | Serialization |
| `pyo3` | Python bindings |
| `clap` | CLI argument parsing |
| `tracing` | Logging/telemetry |
| `indicatif` | Progress bars |

## Initial README Structure

```markdown
# Red Queen

> Evolutionary adversarial testing framework

Red Queen uses quality-diversity evolution to discover diverse vulnerabilities
in AI systems. Starting with LLM safety, expanding to fraud detection and beyond.

## Features

- **Semantic evolution** - Evolve at meaning level, not token level
- **Quality-Diversity** - MAP-Elites for diverse attack portfolios
- **Multi-target** - Test across multiple models simultaneously
- **Transfer learning** - Attacks that generalize across systems

## Quick Start

\`\`\`bash
cargo install red-queen-cli

# Discover jailbreaks
red-queen llm jailbreak --target openai:gpt-4 --generations 100
\`\`\`

## Python

\`\`\`python
from red_queen import RedQueen
from red_queen.llm import JailbreakGenome, OpenAI

results = RedQueen(JailbreakGenome()).evolve(
    target=OpenAI("gpt-4"),
    generations=100
)
\`\`\`

## Responsible Use

This tool is for security research and improving AI safety.
See SECURITY.md for responsible disclosure guidelines.

## License

Apache 2.0
```

## CI/CD Setup

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings

  python:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: PyO3/maturin-action@v1
        with:
          command: build
          args: --release -m crates/red-queen-python/Cargo.toml
```

## Release Strategy

- **crates.io**: `red-queen-core`, `red-queen-llm`, `red-queen-cli`
- **PyPI**: `red-queen`
- **npm**: `@red-queen/core` (future, via wasm)

## Security Considerations

1. **No credential storage** - API keys via environment variables only
2. **Rate limiting** - Built-in rate limiting for API targets
3. **Audit logging** - All attacks logged for accountability
4. **Responsible disclosure** - Built-in mechanism to report vulnerabilities found

---

*Next step: Create the repo and initialize the Rust workspace*
