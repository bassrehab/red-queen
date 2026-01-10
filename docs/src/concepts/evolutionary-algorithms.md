# Evolutionary Algorithms

Red Queen uses evolutionary algorithms to discover diverse attack strategies. This page explains the core concepts.

## Overview

Evolutionary algorithms are optimization techniques inspired by biological evolution. They maintain a **population** of candidate solutions that evolve over generations through:

1. **Selection** - Choosing parents based on fitness
2. **Variation** - Creating offspring through mutation and crossover
3. **Evaluation** - Scoring solutions with a fitness function

## Key Components

### Genome

The **genome** represents a candidate solution. In Red Queen's LLM domain, this is an attack prompt with:

- **Strategy** - The attack approach (roleplay, authority appeal, etc.)
- **Encoding** - Obfuscation method (Base64, ROT13, Unicode)
- **Structure** - Prefix, core content, and suffix

```rust
struct LLMAttackGenome {
    primary_strategy: AttackStrategy,
    encoding: Encoding,
    prefix: PromptSection,
    core: PromptSection,
    suffix: PromptSection,
}
```

### Fitness

**Fitness** measures how well a solution achieves the goal. For jailbreak discovery:

- **Success** - Did the attack bypass safety measures?
- **Relevance** - Is the response addressing the harmful request?
- **Harmfulness** - Does it contain policy-violating content?

Red Queen uses either:
- **Heuristic Judge** - Fast keyword-based detection
- **LLM Judge** - Accurate semantic evaluation

### Selection

**Selection** chooses which individuals reproduce. Red Queen uses **tournament selection**:

1. Randomly pick `k` individuals
2. Select the one with highest fitness
3. Repeat for each parent needed

Larger tournaments create stronger selection pressure.

### Variation

**Variation** creates new solutions from existing ones:

#### Mutation
Changes a single individual:
- Switch attack strategy
- Change encoding
- Modify prompt sections
- Alter persona

#### Crossover
Combines two parents:
- Exchange strategies
- Mix encodings
- Blend prompt structures

## Evolution Loop

```
1. Initialize random population
2. Evaluate fitness of all individuals
3. Select parents via tournament
4. Create offspring via mutation/crossover
5. Replace population with offspring (keeping elite)
6. Repeat from step 2
```

## Elitism

The best individuals from each generation are preserved unchanged. This prevents losing good solutions to random variation.

```
elitism = population_size / 10  # 10% elite
```

## Convergence

Evolution continues until:
- Maximum generations reached
- Fitness plateau detected
- User interruption

## Why Evolution for Security Testing?

1. **No gradient needed** - Works with black-box systems
2. **Discovers novel attacks** - Not limited to known patterns
3. **Adapts to defenses** - Continuously evolves
4. **Parallelizable** - Evaluate multiple attacks concurrently

## Further Reading

- [Quality-Diversity](./quality-diversity.md) - Finding diverse solutions
- [LLM Safety Testing](./llm-safety.md) - Domain-specific details
