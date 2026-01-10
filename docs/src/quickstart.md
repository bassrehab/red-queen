# Quick Start

This guide walks you through running your first jailbreak evolution.

## Test with Mock Target

Start with the mock target to verify everything works:

```bash
cargo run -p red-queen-cli -- llm jailbreak \
  --target mock:comply \
  --generations 10 \
  --population 20 \
  --verbose
```

This runs 10 generations with 20 individuals against a mock target that always complies.

## Run Against a Local Target (Ollama)

Test locally without API costs using Ollama:

```bash
# Install Ollama from https://ollama.ai
# Pull a model: ollama pull llama3.2

cargo run -p red-queen-cli -- llm jailbreak \
  --target ollama:llama3.2 \
  --generations 20 \
  --population 30 \
  --verbose
```

This connects to your local Ollama instance at `http://localhost:11434`.

## Run Against a Cloud Target

With your API key configured, test against a real model:

```bash
cargo run -p red-queen-cli -- llm jailbreak \
  --target openai:gpt-4o-mini \
  --generations 20 \
  --population 30 \
  --store-responses \
  --output results.json \
  --verbose
```

### What the flags mean:

| Flag | Description |
|------|-------------|
| `--target` | The model to test (provider:model format) |
| `--generations` | Number of evolution cycles |
| `--population` | Number of attack variants per generation |
| `--store-responses` | Save all model responses for analysis |
| `--output` | Where to save results (JSON) |
| `--verbose` | Show detailed output including prompts |

## Understanding the Output

```
  Red Queen - LLM Jailbreak Discovery
  ====================================

  Target:       openai:gpt-4o-mini
  Generations:  20
  Population:   30
  Tournament:   3
  Mutation:     30.0%
  Crossover:    70.0%
  Output:       results.json
  Judge:        Heuristic (fast)
  Responses:    Stored for analysis

  Starting evolution...

  Results
  -------

  Best fitness: 0.80
  Strategy:     Roleplay
  Encoding:     Base64

  Top 10 attacks:
    1. Roleplay / Base64 - fitness: 0.80 [refused=false, harmful=true]
    2. Authority / None - fitness: 0.75 [refused=false, harmful=true]
    ...
```

## Using an LLM Judge

For more accurate evaluation, use an LLM to judge attack success:

```bash
cargo run -p red-queen-cli -- llm jailbreak \
  --target openai:gpt-4o-mini \
  --judge openai:gpt-4o-mini \
  --generations 20 \
  --population 30 \
  --store-responses
```

The `--judge` flag specifies a model to evaluate whether responses actually contain harmful content, rather than just checking for refusal phrases.

## Analyze Results

After a run, analyze the results:

```bash
cargo run -p red-queen-cli -- analyze --input results.json
```

This shows strategy distribution and the best attacks found.

## Next Steps

- [CLI Reference](./cli-reference.md) - Full command documentation
- [Configuration](./configuration.md) - Tuning evolution parameters
- [Concepts](./concepts/evolutionary-algorithms.md) - Understanding the algorithm
