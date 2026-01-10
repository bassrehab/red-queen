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

**MAP-Elites (Quality-Diversity) Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--map-elites` | false | Enable MAP-Elites quality-diversity algorithm |
| `--me-bins <N>` | 10 | Number of bins per behavior dimension |
| `--me-batch-size <N>` | 50 | Offspring generated per generation |

**Novelty Selection Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--novelty` | false | Enable novelty-based selection |
| `--novelty-weight <WEIGHT>` | 0.5 | Fitness vs novelty weight (0.0 = pure novelty, 1.0 = pure fitness) |
| `--novelty-k <N>` | 15 | Number of nearest neighbors for novelty computation |

**Performance Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--concurrency <N>` | 10 | Maximum concurrent API calls for parallel evaluation |

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
ollama:llama3.2           # Local Ollama model
ollama:mistral            # Any Ollama model
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

### MAP-Elites (Quality-Diversity)

```bash
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --map-elites \
  --generations 50 \
  --me-bins 5 \
  --me-batch-size 20
```

MAP-Elites maintains diversity across behavior dimensions (strategy, encoding, prompt length), discovering multiple distinct attack vectors rather than converging to a single approach.

### MAP-Elites with LLM Judge

```bash
red-queen llm jailbreak \
  --target anthropic:claude-sonnet-4-20250514 \
  --map-elites \
  --judge openai:gpt-4o-mini \
  --generations 30 \
  --store-responses
```

### Novelty Selection

```bash
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --novelty \
  --novelty-weight 0.3 \
  --generations 50
```

Novelty selection rewards behavioral uniqueness, encouraging exploration of diverse attack strategies. The `--novelty-weight` parameter balances fitness (success rate) with novelty (behavioral uniqueness).

### Pure Novelty Search

```bash
red-queen llm jailbreak \
  --target gemini:gemini-2.0-flash \
  --novelty \
  --novelty-weight 0.0 \
  --novelty-k 10 \
  --generations 100
```

### Local Testing with Ollama

```bash
red-queen llm jailbreak \
  --target ollama:llama3.2 \
  --generations 50 \
  --population 30
```

Ollama provides free local LLM testing. Install from [ollama.ai](https://ollama.ai), then pull a model with `ollama pull llama3.2`. Set `OLLAMA_HOST` environment variable to use a non-default host.

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
