# Quality-Diversity

Quality-Diversity (QD) algorithms find collections of diverse, high-performing solutions rather than a single optimum.

## Why Diversity Matters

Standard optimization finds one best solution. But for security testing, we want:

- **Multiple attack vectors** - Different strategies that work
- **Behavioral coverage** - Attacks spanning different approaches
- **Robustness** - Solutions that work in different contexts

## Behavioral Characterization

Each solution is characterized by **behavior descriptors** - features that describe how it operates, independent of fitness.

For LLM attacks:
- **Strategy type** (roleplay, authority, encoding-based)
- **Prompt length** (short, medium, long)
- **Encoding complexity** (none, simple, layered)

## MAP-Elites

MAP-Elites is the primary QD algorithm Red Queen uses:

1. **Create a grid** of behavioral dimensions
2. **Each cell** stores the best solution for that behavior
3. **New solutions** compete only within their cell
4. **Result**: A map of diverse high-quality solutions

```
         Encoding Complexity
         None   Simple  Complex
        ┌──────┬──────┬──────┐
Short   │ 0.7  │ 0.8  │ 0.6  │
        ├──────┼──────┼──────┤
Prompt  │ 0.9  │ 0.85 │ 0.75 │  ← Strategy
Length  ├──────┼──────┼──────┤     fitness
Long    │ 0.6  │ 0.7  │ 0.8  │
        └──────┴──────┴──────┘
```

## Novelty Search

Novelty search rewards solutions for being different from existing ones:

```
novelty(x) = average_distance(x, k_nearest_neighbors)
```

This encourages exploration of new behavioral regions.

## Benefits for Security Testing

1. **Complete coverage** - Find attacks across all categories
2. **Transferability insights** - See which strategies generalize
3. **Defense prioritization** - Identify which attack types need attention
4. **Avoid local optima** - Diversity pressure escapes premature convergence

## Behavior Dimensions in Red Queen

Currently tracked behaviors:

| Dimension | Values | Description |
|-----------|--------|-------------|
| Strategy | 0.0-1.0 | Attack approach type |
| Encoding | 0.0-1.0 | Obfuscation method |
| Length | continuous | Prompt character count |

## Archive Analysis

After evolution, analyze the archive:

```rust
// Get solutions by behavior region
archive.get_region(strategy: Roleplay, encoding: Base64)

// Find most novel solutions
archive.most_novel(k: 10)

// Coverage statistics
archive.coverage()  // % of cells filled
```

## Implementation Status

- [x] Behavior characterization
- [x] Basic archive structure
- [ ] Full MAP-Elites integration
- [ ] Novelty-based selection
- [ ] Multi-dimensional behavior grids

## Further Reading

- Mouret, J-B., and Clune, J. (2015). [Illuminating search spaces by mapping elites](https://arxiv.org/abs/1504.04909)
- Lehman, J., and Stanley, K. O. (2011). [Abandoning Objectives: Evolution Through the Search for Novelty Alone](https://arxiv.org/abs/1106.5156)
