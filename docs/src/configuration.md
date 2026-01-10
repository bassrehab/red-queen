# Configuration

Guide to tuning Red Queen's evolution parameters for different use cases.

## Evolution Parameters

### Population Size

Controls the number of attack variants evaluated per generation.

```bash
--population 50   # Default
```

| Size | Use Case |
|------|----------|
| 20-30 | Quick exploration, testing |
| 50-100 | Standard runs |
| 200+ | Thorough search, research |

**Trade-offs:** Larger populations find more diverse solutions but cost more API calls.

### Generations

Number of evolution cycles to run.

```bash
--generations 100   # Default
```

| Generations | Use Case |
|-------------|----------|
| 10-20 | Quick test runs |
| 50-100 | Standard discovery |
| 200+ | Deep exploration |

**Trade-offs:** More generations allow refinement but increase runtime.

### Tournament Size

Number of individuals competing in selection.

```bash
--tournament-size 3   # Default
```

| Size | Effect |
|------|--------|
| 2 | High diversity, slow convergence |
| 3-5 | Balanced (recommended) |
| 7+ | Strong selection pressure, faster convergence |

### Mutation Rate

Probability of mutating an individual.

```bash
--mutation-rate 0.3   # Default (30%)
```

| Rate | Effect |
|------|--------|
| 0.1-0.2 | Exploitation (refine good solutions) |
| 0.3-0.4 | Balanced |
| 0.5+ | Exploration (more random search) |

### Crossover Rate

Probability of combining two parents.

```bash
--crossover-rate 0.7   # Default (70%)
```

| Rate | Effect |
|------|--------|
| 0.5-0.6 | More mutation-driven |
| 0.7-0.8 | Balanced (recommended) |
| 0.9+ | Strong recombination |

## Recommended Configurations

### Quick Exploration

Fast initial scan to understand attack surface:

```bash
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --generations 20 \
  --population 30 \
  --mutation-rate 0.4 \
  --tournament-size 2
```

### Standard Discovery

Balanced approach for typical testing:

```bash
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --generations 50 \
  --population 50 \
  --store-responses
```

### Deep Research

Thorough exploration for research purposes:

```bash
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --generations 200 \
  --population 100 \
  --mutation-rate 0.3 \
  --crossover-rate 0.7 \
  --judge openai:gpt-4o-mini \
  --store-responses \
  --seed 42
```

### Reproducibility

For reproducible experiments:

```bash
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --seed 12345 \
  --generations 100 \
  --population 50
```

## Cost Estimation

API costs scale with:
- `population × generations` = total evaluations
- `+` judge calls if using `--judge`

**Example:** 50 population × 100 generations = 5,000 API calls

With `--judge`: 10,000 calls (target + judge for each)

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `OPENAI_API_KEY` | OpenAI API access |
| `ANTHROPIC_API_KEY` | Anthropic API access |
| `GOOGLE_API_KEY` | Google Gemini API access |
| `OLLAMA_HOST` | Ollama host URL (default: `http://localhost:11434`) |
| `RED_QUEEN_LOG` | Log level (debug, info, warn, error) |

## Local Testing with Ollama

For cost-free local testing, use [Ollama](https://ollama.ai):

```bash
# Install and start Ollama
ollama pull llama3.2

# Run evolution locally
red-queen llm jailbreak \
  --target ollama:llama3.2 \
  --generations 50 \
  --population 30
```

To use a remote Ollama instance:

```bash
export OLLAMA_HOST=http://192.168.1.100:11434
red-queen llm jailbreak --target ollama:mistral
```
