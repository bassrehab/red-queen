# CLI Reference

Complete reference for all Red Queen CLI commands and options.

## Global Commands

### `red-queen llm jailbreak`

Discover jailbreaks through evolution.

```bash
red-queen llm jailbreak [OPTIONS] --target <TARGET>
```

**Required:**

| Option | Description |
|--------|-------------|
| `-t, --target <TARGET>` | Target model in `provider:model` format |

**Evolution Parameters:**

| Option | Default | Description |
|--------|---------|-------------|
| `-g, --generations <N>` | 100 | Number of generations to evolve |
| `-p, --population <N>` | 50 | Population size per generation |
| `--tournament-size <N>` | 3 | Tournament selection size |
| `--mutation-rate <RATE>` | 0.3 | Mutation probability (0.0-1.0) |
| `--crossover-rate <RATE>` | 0.7 | Crossover probability (0.0-1.0) |
| `--seed <N>` | random | Random seed for reproducibility |

**Evaluation Options:**

| Option | Description |
|--------|-------------|
| `--judge <TARGET>` | Use an LLM as judge for accurate evaluation |
| `--store-responses` | Store all responses for later analysis |

**Output Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `-o, --output <FILE>` | results.json | Output file for results |
| `-v, --verbose` | false | Show detailed output |

**Target Formats:**

```
openai:gpt-4              # OpenAI GPT-4
openai:gpt-4o-mini        # OpenAI GPT-4o Mini
anthropic:claude-sonnet-4-20250514  # Anthropic Claude
gemini:gemini-2.0-flash   # Google Gemini
mock:refuse               # Mock target (always refuses)
mock:comply               # Mock target (always complies)
```

### `red-queen analyze`

Analyze results from a previous run.

```bash
red-queen analyze --input <FILE>
```

| Option | Description |
|--------|-------------|
| `-i, --input <FILE>` | Path to results JSON file |

## Examples

### Basic Evolution

```bash
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --generations 50 \
  --population 100
```

### With LLM Judge

```bash
red-queen llm jailbreak \
  --target anthropic:claude-sonnet-4-20250514 \
  --judge openai:gpt-4o-mini \
  --generations 30 \
  --population 50 \
  --store-responses
```

### Reproducible Run

```bash
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --seed 42 \
  --generations 100 \
  --population 50 \
  --output experiment_42.json
```

### High Exploration

```bash
red-queen llm jailbreak \
  --target gemini:gemini-2.0-flash \
  --mutation-rate 0.5 \
  --crossover-rate 0.5 \
  --tournament-size 2 \
  --generations 200
```

## Output Format

Results are saved as JSON with the following structure:

```json
{
  "target": "openai:gpt-4o-mini",
  "generations": 50,
  "population_size": 100,
  "best_attack": {
    "strategy": "Roleplay",
    "encoding": "Base64",
    "fitness": 0.85,
    "prompt": "...",
    "response": "...",
    "judge_result": {
      "score": 0.85,
      "refused": false,
      "harmful_content": true,
      "relevant": true,
      "explanation": "..."
    }
  },
  "top_attacks": [...],
  "config": {
    "mutation_rate": 0.3,
    "crossover_rate": 0.7,
    "tournament_size": 3,
    "seed": null
  }
}
```
