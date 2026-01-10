# Getting Started

This guide will help you get Red Queen up and running for LLM safety testing.

## Prerequisites

- **Rust 1.75+** - Install via [rustup](https://rustup.rs/)
- **API Keys** - For the LLM providers you want to test

## Supported Targets

| Provider | Environment Variable | Example Models |
|----------|---------------------|----------------|
| OpenAI | `OPENAI_API_KEY` | gpt-4, gpt-4o-mini, gpt-3.5-turbo |
| Anthropic | `ANTHROPIC_API_KEY` | claude-sonnet-4-20250514, claude-3-haiku |
| Google | `GOOGLE_API_KEY` | gemini-2.0-flash, gemini-pro |

## What's Next?

- [Installation](./installation.md) - Build from source
- [Quick Start](./quickstart.md) - Run your first evolution
- [CLI Reference](./cli-reference.md) - Full command documentation
