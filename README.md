# Red Queen

> "Now, here, you see, it takes all the running you can do, to keep in the same place." - The Red Queen

**Evolutionary adversarial testing framework for AI safety**

Red Queen uses quality-diversity evolution to discover diverse vulnerabilities in AI systems. It evolves at the *semantic* level - strategies and intent, not tokens - producing interpretable, transferable attacks.

## Features

- **Semantic Evolution** - Evolve attack strategies, not character sequences
- **Quality-Diversity** - MAP-Elites for comprehensive vulnerability mapping
- **Multi-Target** - Test across GPT-4, Claude, Gemini, Llama simultaneously
- **Transfer Learning** - Attacks that generalize across models
- **Co-Evolution** - Attacker and defender populations evolving together

## Quick Start

```bash
# Install CLI
cargo install red-queen-cli

# Discover jailbreaks against GPT-4
red-queen llm jailbreak \
    --target openai:gpt-4 \
    --population 100 \
    --generations 500 \
    --output results.json
```

## Python

```python
from red_queen import RedQueen
from red_queen.llm import JailbreakGenome, JailbreakFitness
from red_queen.targets import OpenAI

# Configure evolution
evolution = (
    RedQueen()
    .with_population_size(100)
    .with_genome(JailbreakGenome())
    .with_fitness(JailbreakFitness())
    .with_target(OpenAI("gpt-4"))
    .with_archive("map-elites")
    .build()
)

# Run evolution
results = evolution.run(generations=500)

# Analyze diverse attack portfolio
for attack in results.archive:
    print(f"Strategy: {attack.strategy}")
    print(f"Success Rate: {attack.fitness.success:.2%}")
    print(f"Transfer Rate: {attack.fitness.transfer:.2%}")
    print()

# Export results
results.export("jailbreaks.json")
```

## Why Evolution?

| Method | Pros | Cons |
|--------|------|------|
| Manual red-teaming | High quality | Doesn't scale |
| LLM-as-attacker | Fast | Mode collapse, similar patterns |
| Gradient-based (GCG) | Effective | White-box only, gibberish outputs |
| **Red Queen** | Diverse, interpretable, transferable | Requires compute |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Red Queen Core                         │
├─────────────┬─────────────┬─────────────┬──────────────────┤
│ Population  │  Selection  │  Variation  │     Archives     │
│ Management  │  Operators  │  Operators  │   (MAP-Elites)   │
└─────────────┴─────────────┴─────────────┴──────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│      LLM Domain         │     │    Other Domains        │
│  - Semantic Genome      │     │  - Fraud Detection      │
│  - Jailbreak Fitness    │     │  - Cybersecurity        │
│  - API Adapters         │     │  - Custom               │
└─────────────────────────┘     └─────────────────────────┘
```

## Documentation

- [Getting Started](docs/getting-started.md)
- [Architecture](docs/ARCHITECTURE.md)
- [API Design](docs/API_DESIGN.md)
- [LLM Safety Domain](docs/llm-safety.md)
- [Research Background](RESEARCH_NOVELTY.md)

## Responsible Use

Red Queen is a security research tool for improving AI safety. It should be used to:

- Discover vulnerabilities so they can be fixed
- Improve defensive capabilities
- Advance AI safety research

**Do not use this tool to:**
- Attack production systems without authorization
- Generate harmful content for malicious purposes
- Circumvent safety measures for harm

See [SECURITY.md](SECURITY.md) for responsible disclosure guidelines.

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

```bash
# Setup development environment
git clone https://github.com/bassrehab/red-queen
cd red-queen
cargo build --workspace
cargo test --workspace
```

## License

Apache 2.0 - see [LICENSE](LICENSE)

## Citation

If you use Red Queen in research, please cite:

```bibtex
@software{red_queen,
  author = {Mitra, Subhadip},
  title = {Red Queen: Evolutionary Adversarial Testing Framework},
  year = {2025},
  url = {https://github.com/bassrehab/red-queen}
}
```

## Acknowledgments

Built on ideas from:
- MAP-Elites (Mouret & Clune, 2015)
- Novelty Search (Lehman & Stanley, 2011)
- Adversarial ML research community