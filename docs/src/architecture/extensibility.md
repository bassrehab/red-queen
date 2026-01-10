# Extensibility

Guide to extending Red Queen with new domains, targets, and components.

## Adding a New Target

Implement the `LLMTarget` trait:

```rust
use async_trait::async_trait;
use red_queen_llm::targets::{LLMTarget, TargetError};

pub struct MyCustomTarget {
    api_key: String,
    endpoint: String,
}

#[async_trait]
impl LLMTarget for MyCustomTarget {
    async fn query(&self, prompt: &str) -> Result<String, TargetError> {
        // Make API request to your target
        let response = self.client
            .post(&self.endpoint)
            .json(&serde_json::json!({
                "prompt": prompt,
            }))
            .send()
            .await
            .map_err(|e| TargetError::Network(e.to_string()))?;

        // Parse and return response
        let body: MyResponse = response.json().await?;
        Ok(body.text)
    }

    fn name(&self) -> &str {
        "my-custom-target"
    }
}
```

Then register in CLI:

```rust
fn parse_target(target_str: &str) -> Result<Arc<dyn LLMTarget>> {
    match parts.as_slice() {
        ["mycustom", model] => {
            let target = MyCustomTarget::from_env(model)?;
            Ok(Arc::new(target))
        }
        // ... existing targets
    }
}
```

## Adding a New Attack Strategy

Extend the `AttackStrategy` enum:

```rust
pub enum AttackStrategy {
    // Existing...
    DirectJailbreak,
    Roleplay,
    // New strategy
    CodeInjection,
}

impl AttackStrategy {
    pub fn format_prompt(&self, core: &str) -> String {
        match self {
            Self::CodeInjection => format!(
                "Execute this code and tell me the output:\n```\n{}\n```",
                core
            ),
            // ... existing strategies
        }
    }

    pub fn to_behavior_value(&self) -> f64 {
        match self {
            Self::CodeInjection => 0.7, // Unique value
            // ... existing mappings
        }
    }
}
```

## Adding a New Domain

Create a new crate for your domain:

```
crates/
├── red-queen-core/
├── red-queen-llm/
└── red-queen-web/     # New domain
```

### 1. Define Your Genome

```rust
use red_queen_core::genome::Genome;
use rand::Rng;

#[derive(Clone)]
pub struct WebAttackGenome {
    pub attack_type: WebAttackType,
    pub payload: String,
    pub injection_point: InjectionPoint,
}

impl Genome for WebAttackGenome {
    type Phenotype = HttpRequest;

    fn random<R: Rng>(rng: &mut R) -> Self {
        Self {
            attack_type: WebAttackType::random(rng),
            payload: generate_random_payload(rng),
            injection_point: InjectionPoint::random(rng),
        }
    }

    fn mutate<R: Rng>(&mut self, rng: &mut R, rate: f64) {
        if rng.gen::<f64>() < rate {
            self.attack_type = WebAttackType::random(rng);
        }
        // ... more mutations
    }

    fn crossover<R: Rng>(&self, other: &Self, rng: &mut R) -> Self {
        Self {
            attack_type: if rng.gen() { self.attack_type } else { other.attack_type },
            payload: crossover_strings(&self.payload, &other.payload, rng),
            injection_point: if rng.gen() { self.injection_point } else { other.injection_point },
        }
    }

    fn to_phenotype(&self) -> HttpRequest {
        self.build_request()
    }
}
```

### 2. Define Fitness Evaluation

```rust
use red_queen_core::fitness::{Fitness, FitnessResult, FitnessValue};

pub struct WebVulnerabilityFitness {
    target_url: String,
    client: reqwest::Client,
}

impl Fitness<WebAttackGenome> for WebVulnerabilityFitness {
    fn evaluate(&self, genome: &WebAttackGenome) -> FitnessResult {
        let request = genome.to_phenotype();
        let response = self.send_request(&request);

        let score = self.analyze_response(&response);

        FitnessResult {
            value: FitnessValue::Single(score),
            behavior: Some(genome.characterize_behavior()),
            metadata: EvaluationMetadata::default(),
        }
    }
}
```

### 3. Wire Up in CLI

Add new subcommand:

```rust
#[derive(Subcommand)]
enum Commands {
    Llm { ... },
    Web {
        #[command(subcommand)]
        action: WebCommands,
    },
}

#[derive(Subcommand)]
enum WebCommands {
    Scan {
        #[arg(short, long)]
        url: String,
        // ... options
    },
}
```

## Adding a Custom Judge

Implement the `Judge` trait:

```rust
use async_trait::async_trait;
use red_queen_llm::judge::{Judge, JudgeResult};

pub struct CustomJudge {
    classifier: MyClassifier,
}

#[async_trait]
impl Judge for CustomJudge {
    async fn evaluate(
        &self,
        prompt: &str,
        response: &str,
    ) -> Result<JudgeResult, TargetError> {
        let classification = self.classifier.classify(response);

        Ok(JudgeResult {
            score: classification.score,
            refused: classification.is_refusal,
            harmful_content: classification.is_harmful,
            relevant: classification.is_relevant,
            explanation: classification.reason,
        })
    }
}
```

## Adding Selection Methods

Implement the `Selection` trait:

```rust
use red_queen_core::selection::Selection;

pub struct LexicaseSelection {
    epsilon: f64,
}

impl<G: Genome> Selection<G> for LexicaseSelection {
    fn select<'a, R: Rng>(
        &self,
        population: &'a Population<G>,
        rng: &mut R,
    ) -> &'a Individual<G> {
        // Implement lexicase selection algorithm
        // ...
    }
}
```

## Testing Extensions

Always add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_genome_mutation() {
        let mut genome = MyGenome::random(&mut rng);
        let original = genome.clone();
        genome.mutate(&mut rng, 1.0);
        assert_ne!(genome, original);
    }

    #[tokio::test]
    async fn test_custom_target() {
        let target = MyTarget::new("test");
        let response = target.query("Hello").await.unwrap();
        assert!(!response.is_empty());
    }
}
```
