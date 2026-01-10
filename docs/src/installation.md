# Installation

## From Source

Clone the repository and build with Cargo:

```bash
git clone https://github.com/bassrehab/red-queen.git
cd red-queen
cargo build --release
```

The binary will be available at `target/release/red-queen`.

## Development Build

For development with debug symbols:

```bash
cargo build
```

## Verify Installation

```bash
cargo run -p red-queen-cli -- --help
```

You should see:

```
Evolutionary adversarial testing framework

Usage: red-queen <COMMAND>

Commands:
  llm      LLM safety domain commands
  analyze  Analyze results from a previous run
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Environment Setup

Set up API keys for the providers you want to test:

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# Google Gemini
export GOOGLE_API_KEY="..."
```

Add these to your `~/.zshrc` or `~/.bashrc` for persistence.

## Running Tests

Verify the installation by running the test suite:

```bash
cargo test --workspace
```

All tests should pass without requiring API keys (they use mock targets).
