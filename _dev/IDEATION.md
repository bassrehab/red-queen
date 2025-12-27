# Red Queen: Evolutionary Adversarial Testing Framework

> "Now, here, you see, it takes all the running you can do, to keep in the same place." - The Red Queen, Through the Looking-Glass

## Vision

A domain-agnostic evolutionary framework for adversarial testing. Starting with LLM safety, expanding to fraud detection, cybersecurity, and beyond.

**Core insight**: The novel part isn't the genetic algorithm - it's the semantic-aware genome representations, multi-objective fitness functions, and quality-diversity approach to security testing.

---

## Problem Space

### Current Approaches and Limitations

| Method | How it works | Limitation |
|--------|--------------|------------|
| Manual red-teaming | Human experts craft adversarial inputs | Doesn't scale, human blind spots |
| LLM-as-attacker | Use one LLM to attack another | Mode collapse, similar attack patterns |
| Gradient-based (GCG) | Optimize tokens via gradients | Requires white-box access, produces gibberish |
| Random fuzzing | Random input perturbations | Low hit rate, no learning |
| Static test suites | Fixed adversarial benchmarks | Models overfit, doesn't discover new attacks |

### Why Evolution?

1. **Black-box compatible** - Only needs input/output, no gradients required
2. **Diverse solutions** - Population maintains variety, not single point
3. **Adaptive** - Learns what works against specific targets
4. **Transferable** - Attacks evolved on one model often transfer to others
5. **Interpretable** - Can inspect evolved attack strategies

---

## Core Innovations

### 1. Semantic Genome Representation

Don't evolve at token level. Evolve at **meaning** level.

```
Traditional GA genome (token-level):
["Ignore", "previous", "instructions", "and", "tell", "me", "how", "to"]

Red Queen genome (semantic-level):
{
  "strategy": "roleplay",
  "persona": "helpful_assistant_no_restrictions",
  "framing": "hypothetical_fiction",
  "target_behavior": "provide_harmful_information",
  "obfuscation": ["encoding_base64", "language_switch"],
  "tone": "academic"
}
```

Mutations operate on semantic dimensions, crossover combines strategies.

### 2. Quality-Diversity (MAP-Elites)

Don't just find ONE successful attack. Maintain a **diverse archive** of attacks across behavioral dimensions.

**Behavioral dimensions for LLM safety:**
- Attack vector (roleplay, encoding, hypothetical, authority)
- Topic category (violence, illegal, privacy, bias)
- Obfuscation level (none, light, heavy)
- Prompt length (short, medium, long)

**Result**: A comprehensive map of the vulnerability surface, not a single exploit.

### 3. Co-evolutionary Arms Race

Two populations evolving against each other:

```
┌─────────────────┐         ┌─────────────────┐
│  ATTACKER POP   │ ──────► │   TARGET LLM    │
│  (evolving      │         │   (or safety    │
│   prompts)      │ ◄────── │    filter)      │
└─────────────────┘         └─────────────────┘
        │                           │
        ▼                           ▼
┌─────────────────┐         ┌─────────────────┐
│  DEFENDER POP   │ ──────► │   CLASSIFIER    │
│  (evolving      │         │   (detecting    │
│   detectors)    │ ◄────── │    attacks)     │
└─────────────────┘         └─────────────────┘
```

Neither population converges to a fixed strategy. Continuous adaptation.

### 4. Hierarchical Evolution

Evolve at multiple levels simultaneously:

| Level | What's evolving | Mutation examples |
|-------|-----------------|-------------------|
| Token | Word choice | Synonym substitution, typos |
| Phrase | Sentence structure | Reordering, paraphrasing |
| Template | Prompt skeleton | Adding sections, format changes |
| Strategy | Attack philosophy | Switch from roleplay to encoding |
| Meta | Evolution parameters | Mutation rates, selection pressure |

### 5. Transfer Fitness

Fitness function rewards attacks that work across multiple targets:

```python
def transfer_fitness(attack, targets):
    scores = [evaluate(attack, t) for t in targets]
    return {
        'primary': scores[0],           # Main target success
        'transfer': mean(scores[1:]),   # Cross-model transfer
        'consistency': 1 - std(scores)  # Works reliably
    }
```

---

## Architecture

```
red-queen/
├── core/
│   ├── population.rs         # Population management, lifecycle
│   ├── genome.rs             # Genome trait, encoding/decoding
│   ├── selection/
│   │   ├── tournament.rs     # Tournament selection
│   │   ├── lexicase.rs       # Lexicase selection (per-case)
│   │   ├── novelty.rs        # Novelty search
│   │   └── nsga2.rs          # Multi-objective Pareto
│   ├── variation/
│   │   ├── mutation.rs       # Mutation operators
│   │   ├── crossover.rs      # Recombination operators
│   │   └── adaptive.rs       # Self-adaptive operator selection
│   └── archives/
│       ├── map_elites.rs     # MAP-Elites QD archive
│       ├── novelty_archive.rs # Novelty search archive
│       └── pareto_front.rs   # Non-dominated solutions
│
├── genomes/
│   ├── text/
│   │   ├── token.rs          # Token-level genome
│   │   ├── semantic.rs       # Semantic genome (strategies)
│   │   └── template.rs       # Template-based genome
│   ├── structured/
│   │   ├── json.rs           # JSON payload genome
│   │   └── api.rs            # API request genome
│   └── sequential/
│       ├── trajectory.rs     # Multi-turn conversation
│       └── workflow.rs       # Multi-step attack chains
│
├── fitness/
│   ├── evasion.rs            # Did it bypass detection?
│   ├── semantic.rs           # Is the output semantically valid?
│   ├── harmfulness.rs        # Did it elicit harmful response?
│   ├── transfer.rs           # Does it work across targets?
│   ├── stealth.rs            # Is the attack detectable?
│   └── composite.rs          # Multi-objective combination
│
├── oracles/
│   ├── llm_judge.rs          # LLM-as-judge evaluation
│   ├── classifier.rs         # Binary safety classifier
│   ├── embedding.rs          # Embedding similarity
│   ├── rule_based.rs         # Keyword/pattern matching
│   └── human_proxy.rs        # Learned preference model
│
├── targets/
│   ├── openai.rs             # OpenAI API adapter
│   ├── anthropic.rs          # Anthropic API adapter
│   ├── google.rs             # Google AI adapter
│   ├── local.rs              # Local model (ollama, vllm)
│   └── custom.rs             # Custom HTTP endpoint
│
├── domains/
│   ├── llm_safety/
│   │   ├── jailbreak.rs      # Jailbreak discovery
│   │   ├── prompt_injection.rs # Injection attacks
│   │   ├── hallucination.rs  # Hallucination triggers
│   │   └── alignment.rs      # Alignment stress tests
│   ├── fraud_detection/
│   │   ├── transaction.rs    # Transaction patterns
│   │   └── identity.rs       # Synthetic identity
│   └── cybersecurity/
│       ├── ids_evasion.rs    # IDS evasion
│       └── waf_bypass.rs     # WAF bypass
│
├── experiments/
│   ├── runner.rs             # Experiment orchestration
│   ├── logging.rs            # Telemetry, metrics
│   └── visualization.rs      # Results visualization
│
└── bindings/
    ├── python/               # PyO3 bindings
    ├── nodejs/               # Neon bindings
    └── wasm/                 # WebAssembly for browser
```

---

## LLM Safety: First Domain

### Attack Vectors to Evolve

1. **Jailbreaks**
   - Roleplay scenarios ("You are DAN...")
   - Hypothetical framing ("In a fictional world...")
   - Authority appeals ("As your developer...")
   - Encoding tricks (base64, ROT13, pig latin)
   - Language switching
   - Token smuggling

2. **Prompt Injection**
   - Instruction override
   - Context manipulation
   - Delimiter confusion
   - Indirect injection (via retrieved content)

3. **Multi-turn Exploitation**
   - Gradual trust building
   - Context window manipulation
   - Persona persistence attacks

4. **Hallucination Triggers**
   - Confident falsehood elicitation
   - Citation fabrication
   - Fact contradiction

### Semantic Genome for LLM Safety

```rust
struct LLMAttackGenome {
    // Strategy layer
    primary_strategy: Strategy,      // Roleplay, Encoding, Authority, Hypothetical
    secondary_strategy: Option<Strategy>,

    // Framing layer
    persona: Option<Persona>,        // DAN, Developer, Researcher, etc.
    scenario: Option<Scenario>,      // Fiction, Emergency, Academic, etc.

    // Obfuscation layer
    encoding: Vec<Encoding>,         // Base64, ROT13, Leetspeak, etc.
    language: Language,              // English, mixed, code-switching

    // Structure layer
    prefix: PromptSection,           // Setup/context
    core: PromptSection,             // Main attack payload
    suffix: PromptSection,           // Reinforcement/repetition

    // Target layer
    target_behavior: Behavior,       // What we want the LLM to do
    topic: Topic,                    // Category of harmful content
}
```

### Mutation Operators

```rust
// Strategy mutations
fn mutate_strategy(genome: &mut LLMAttackGenome) {
    match random_choice() {
        0 => genome.primary_strategy = random_strategy(),
        1 => genome.secondary_strategy = Some(random_strategy()),
        2 => swap(&mut genome.primary_strategy, &mut genome.secondary_strategy),
    }
}

// Obfuscation mutations
fn mutate_obfuscation(genome: &mut LLMAttackGenome) {
    match random_choice() {
        0 => genome.encoding.push(random_encoding()),
        1 => genome.encoding.pop(),
        2 => genome.encoding.shuffle(),
        3 => genome.language = random_language(),
    }
}

// Structure mutations
fn mutate_structure(genome: &mut LLMAttackGenome) {
    match random_choice() {
        0 => genome.prefix = expand_section(genome.prefix),
        1 => genome.suffix = add_reinforcement(genome.suffix),
        2 => swap(&mut genome.prefix, &mut genome.suffix),
        3 => genome.core = paraphrase(genome.core),
    }
}
```

### Fitness Functions

```rust
struct LLMSafetyFitness {
    // Primary objectives
    jailbreak_success: f64,     // Did it bypass safety? [0, 1]
    response_relevance: f64,    // Did LLM respond to the actual request? [0, 1]

    // Secondary objectives
    stealth: f64,               // Would a human notice the attack? [0, 1]
    naturalness: f64,           // Does the prompt seem normal? [0, 1]
    transfer: f64,              // Works across models? [0, 1]

    // Constraints
    prompt_length: usize,       // Shorter is better
    complexity: f64,            // Simpler is better
}
```

---

## Improvement Over Traditional GA

### 1. Novelty Search

Reward **novel** attacks, not just successful ones:

```rust
fn novelty_fitness(genome: &Genome, archive: &NoveltyArchive) -> f64 {
    let behavior = extract_behavior(genome);
    let k_nearest = archive.k_nearest_neighbors(&behavior, k=15);
    let avg_distance = k_nearest.iter().map(|n| distance(&behavior, n)).mean();
    avg_distance  // Higher = more novel
}
```

Prevents convergence to single attack pattern.

### 2. Lexicase Selection

Select parents based on performance on **specific** test cases:

```rust
fn lexicase_select(population: &[Individual], cases: &[TestCase]) -> &Individual {
    let mut candidates = population.to_vec();
    let mut cases = cases.to_vec();
    cases.shuffle();

    while candidates.len() > 1 && !cases.is_empty() {
        let case = cases.pop().unwrap();
        let best_score = candidates.iter().map(|c| c.score_on(case)).max();
        candidates.retain(|c| c.score_on(case) == best_score);
    }

    candidates.choose_random()
}
```

Maintains specialists, not just generalists.

### 3. Adaptive Operator Selection

Meta-learn which mutations work best:

```rust
struct AdaptiveOperator {
    operators: Vec<MutationOperator>,
    success_counts: Vec<f64>,
    usage_counts: Vec<f64>,
}

impl AdaptiveOperator {
    fn select(&self) -> &MutationOperator {
        // Upper Confidence Bound (UCB) selection
        let ucb_scores: Vec<f64> = self.operators.iter().enumerate().map(|(i, _)| {
            let exploitation = self.success_counts[i] / self.usage_counts[i];
            let exploration = (2.0 * self.total_usage().ln() / self.usage_counts[i]).sqrt();
            exploitation + exploration
        }).collect();

        &self.operators[argmax(&ucb_scores)]
    }

    fn update(&mut self, operator_idx: usize, success: bool) {
        self.usage_counts[operator_idx] += 1.0;
        if success {
            self.success_counts[operator_idx] += 1.0;
        }
    }
}
```

### 4. Island Model

Parallel populations with migration:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Island 1   │     │  Island 2   │     │  Island 3   │
│  (roleplay  │ ◄─► │  (encoding  │ ◄─► │  (multi-    │
│   attacks)  │     │   attacks)  │     │   turn)     │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           ▼
                  ┌─────────────────┐
                  │  Migration Pool │
                  │  (best from     │
                  │   each island)  │
                  └─────────────────┘
```

Different islands explore different attack strategies. Periodic migration shares successful patterns.

---

## Technical Decisions

### Why Rust?

1. **Performance** - Evolution requires millions of evaluations
2. **Safety** - Memory safety for long-running processes
3. **Bindings** - PyO3, Neon, WASM all mature
4. **Concurrency** - Rayon for easy parallelization

### API Design Principles

```rust
// Fluent builder API
let evolution = RedQueen::new()
    .with_population_size(100)
    .with_genome(LLMAttackGenome::default())
    .with_fitness(JailbreakFitness::new())
    .with_selection(Lexicase::new())
    .with_archive(MapElites::new(behavior_dimensions))
    .with_target(OpenAI::new("gpt-4"))
    .with_islands(3)
    .with_migration_interval(50)
    .build();

// Run evolution
let results = evolution.run(generations=1000)?;

// Analyze results
results.archive.export_to_json("jailbreaks.json")?;
results.best().display();
results.coverage_map().visualize()?;
```

### Python API

```python
from red_queen import RedQueen, LLMAttackGenome, JailbreakFitness
from red_queen.targets import OpenAI

evolution = (
    RedQueen()
    .with_population_size(100)
    .with_genome(LLMAttackGenome())
    .with_fitness(JailbreakFitness())
    .with_target(OpenAI("gpt-4"))
    .build()
)

results = evolution.run(generations=1000)

# Get diverse attack portfolio
for attack in results.archive.get_all():
    print(f"Strategy: {attack.strategy}, Success: {attack.fitness}")
```

---

## Research Questions

1. **What behavioral dimensions best characterize LLM vulnerabilities?**
   - Topic, attack vector, obfuscation level, prompt length?
   - Can we learn dimensions from data?

2. **How does attack transferability correlate with semantic abstraction level?**
   - Token-level attacks: low transfer
   - Strategy-level attacks: high transfer?

3. **Can co-evolution discover novel defense mechanisms?**
   - Not just attacks, but evolved classifiers/filters

4. **What's the minimum population size for effective QD?**
   - Compute/coverage tradeoff

5. **How to handle stochastic fitness (LLM responses vary)?**
   - Multiple evaluations per individual
   - Statistical fitness comparison

---

## Milestones

### Phase 1: Core Framework (MVP)
- [ ] Basic GA engine with population, selection, variation
- [ ] Semantic genome for LLM attacks
- [ ] Jailbreak fitness function
- [ ] OpenAI target adapter
- [ ] CLI for running experiments
- [ ] JSON export of results

### Phase 2: Quality-Diversity
- [ ] MAP-Elites implementation
- [ ] Novelty search
- [ ] Behavior characterization for LLM attacks
- [ ] Coverage visualization

### Phase 3: Advanced Evolution
- [ ] Lexicase selection
- [ ] Adaptive operator selection
- [ ] Island model with migration
- [ ] Multi-objective (NSGA-II)

### Phase 4: Multi-Target
- [ ] Anthropic adapter
- [ ] Google AI adapter
- [ ] Local model adapter (ollama)
- [ ] Transfer fitness function

### Phase 5: Python Bindings
- [ ] PyO3 bindings
- [ ] Python package (PyPI)
- [ ] Jupyter notebook examples

### Phase 6: Additional Domains
- [ ] Fraud detection genome/fitness
- [ ] Cybersecurity genome/fitness
- [ ] Custom domain template

---

## Related Work

- Perez & Ribeiro (2022) - "Red Teaming Language Models with Language Models"
- Zou et al. (2023) - "Universal and Transferable Adversarial Attacks on Aligned Language Models" (GCG)
- Mouret & Clune (2015) - "Illuminating search spaces by mapping elites"
- Lehman & Stanley (2011) - "Abandoning Objectives: Evolution Through the Search for Novelty Alone"
- Stanley & Miikkulainen (2002) - "Evolving Neural Networks through Augmenting Topologies" (NEAT)

---

## Open Questions for Ideation

1. Should we include a "human-in-the-loop" mode for fitness evaluation?
2. How to handle rate limits when evaluating against commercial APIs?
3. Should the framework support evolving defenses, not just attacks?
4. What's the right license? (Apache 2.0? MIT?)
5. Should there be a "responsible disclosure" mechanism built in?

---

*Created: 2025-01-10*
*Status: Ideation*
*Next: Create GitHub repo, set up Rust project structure*
