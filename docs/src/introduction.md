# Red Queen

**Evolutionary Adversarial Testing Framework**

Red Queen is a quality-diversity evolution framework for discovering diverse vulnerabilities in AI systems. Named after the Red Queen hypothesis in evolutionary biology, it continuously evolves attack strategies to keep pace with improving AI safety measures.

## Why Red Queen?

Traditional adversarial testing approaches have limitations:

- **Manual red-teaming** doesn't scale and misses edge cases
- **Static benchmarks** become outdated as models improve
- **Random fuzzing** lacks the semantic understanding needed for LLMs
- **Single-objective optimization** finds one attack but misses diverse failure modes

Red Queen addresses these by:

1. **Semantic Genome Representation** - Operating at the meaning level, not token level
2. **Quality-Diversity Evolution** - Finding diverse attacks across behavioral dimensions
3. **Multi-Objective Fitness** - Balancing success, transferability, and stealth
4. **Continuous Adaptation** - Evolving alongside improving defenses

## Core Philosophy

> "It takes all the running you can do, to keep in the same place." - The Red Queen

The novel contribution isn't the genetic algorithm itself - it's the application of quality-diversity evolution to security testing with semantic-aware representations.

## Features

- **Evolutionary Core** - Population-based search with selection, mutation, and crossover
- **LLM Safety Domain** - Specialized genome for jailbreak discovery
- **Multiple Targets** - OpenAI, Anthropic, Google Gemini support
- **LLM Judge** - Accurate attack success evaluation
- **Response Storage** - Full audit trail for analysis
- **Extensible Architecture** - Add new domains and targets easily

## Quick Example

```bash
# Discover jailbreaks with response storage
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --generations 50 \
  --population 100 \
  --store-responses \
  --output results.json
```

## Project Status

Red Queen is currently in active development. The core evolution engine and LLM safety domain are functional, with quality-diversity (MAP-Elites) and advanced features in progress.

## License

MIT License - See [LICENSE](https://github.com/bassrehab/red-queen/blob/main/LICENSE) for details.
