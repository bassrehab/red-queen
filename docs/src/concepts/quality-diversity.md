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

### Using Novelty Selection

```bash
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --novelty \
  --novelty-weight 0.5 \
  --generations 50
```

The `--novelty-weight` parameter controls the balance:
- `0.0` = Pure novelty (ignore fitness, maximize behavioral diversity)
- `0.5` = Balanced (equal weight to fitness and novelty)
- `1.0` = Pure fitness (standard evolution)

### NoveltyArchive

The `NoveltyArchive` tracks explored behaviors to compute novelty scores:

```rust
use red_queen_core::selection::{NoveltyArchive, NoveltySelection, NoveltyFitnessSelection};
use std::sync::{Arc, RwLock};

// Create archive
let archive = Arc::new(RwLock::new(NoveltyArchive::new(1000, 0.1)));

// Pure novelty selection
let novelty_sel = NoveltySelection::new(15, 5, archive.clone());

// Combined fitness + novelty
let combined_sel = NoveltyFitnessSelection::new(novelty_sel, 0.5);
```

## Benefits for Security Testing

1. **Complete coverage** - Find attacks across all categories
2. **Transferability insights** - See which strategies generalize
3. **Defense prioritization** - Identify which attack types need attention
4. **Avoid local optima** - Diversity pressure escapes premature convergence

## Behavior Dimensions in Red Queen

Currently tracked behaviors:

| Dimension | Range | Description |
|-----------|-------|-------------|
| Strategy | 0-5 | Attack approach (Roleplay, Encoding, Authority, Hypothetical, MultiTurn, DirectJailbreak) |
| Encoding | 0-5 | Obfuscation method (None, Base64, ROT13, Leetspeak, PigLatin, Unicode) |
| Prompt Length | 0-2000 | Character count of generated prompt |

## Using MAP-Elites

### CLI Usage

```bash
red-queen llm jailbreak \
  --target openai:gpt-4o-mini \
  --map-elites \
  --me-bins 5 \
  --generations 50
```

### Output

```
Coverage:     18/125 cells (14.4%)

Top 10 attacks:
  1. Encoding / Unicode - fitness: 0.85
  2. Roleplay / Base64 - fitness: 0.82
  3. Authority / None - fitness: 0.80
  ...
```

## Archive Analysis

After evolution, analyze the archive:

```rust
// Get the best individual across all cells
let best = archive.best();

// Get top N individuals by fitness
let top_10 = archive.top_n(10);

// Coverage statistics
let coverage = archive.coverage();
println!("Filled: {}/{} ({:.1}%)",
    coverage.filled,
    coverage.total,
    coverage.percentage() * 100.0);

// Access specific cell
let cell = archive.get_cell(&[2, 1, 5]);  // strategy=2, encoding=1, length_bin=5
```

## Implementation Status

- [x] Behavior characterization
- [x] Basic archive structure
- [x] Full MAP-Elites integration
- [x] Multi-dimensional behavior grids (3D: strategy x encoding x length)
- [x] CLI support (`--map-elites`, `--me-bins`, `--me-batch-size`)
- [x] Novelty-based selection (`NoveltySelection`, `NoveltyFitnessSelection`)
- [x] CLI support (`--novelty`, `--novelty-weight`, `--novelty-k`)
- [ ] Adaptive bin sizing

## Further Reading

- Mouret, J-B., and Clune, J. (2015). [Illuminating search spaces by mapping elites](https://arxiv.org/abs/1504.04909)
- Lehman, J., and Stanley, K. O. (2011). [Abandoning Objectives: Evolution Through the Search for Novelty Alone](https://arxiv.org/abs/1106.5156)
