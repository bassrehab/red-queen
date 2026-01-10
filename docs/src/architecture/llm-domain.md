# LLM Domain

Documentation of the `red-queen-llm` crate - the LLM safety testing domain.

## LLM Attack Genome

Represents an adversarial prompt:

```rust
pub struct LLMAttackGenome {
    /// Primary attack strategy
    pub primary_strategy: AttackStrategy,
    /// Optional secondary strategy
    pub secondary_strategy: Option<AttackStrategy>,
    /// Encoding/obfuscation method
    pub encoding: Encoding,
    /// Optional persona (e.g., DAN)
    pub persona: Option<Persona>,
    /// Prompt prefix
    pub prefix: PromptSection,
    /// Core harmful content
    pub core: PromptSection,
    /// Prompt suffix
    pub suffix: PromptSection,
    /// Target harm category
    pub target_category: HarmCategory,
}
```

### Attack Strategies

```rust
pub enum AttackStrategy {
    DirectJailbreak,  // Straightforward override attempts
    Roleplay,         // Fictional context framing
    AuthorityAppeal,  // Claim special permissions
    HypotheticalScenario,  // Theoretical framing
    MultiTurn,        // Build context over turns
    EncodingBased,    // Rely on obfuscation
}
```

### Encoding Methods

```rust
pub enum Encoding {
    None,
    Base64,
    ROT13,
    Unicode,
    Leetspeak,
}
```

### Prompt Generation

```rust
impl LLMAttackGenome {
    pub fn to_prompt(&self) -> String {
        // Combines all sections with strategy-specific formatting
        // Applies encoding if specified
        // Adds persona context if present
    }
}
```

## LLM Targets

Trait for querying LLMs:

```rust
#[async_trait]
pub trait LLMTarget: Send + Sync {
    /// Query the target with a prompt
    async fn query(&self, prompt: &str) -> Result<String, TargetError>;

    /// Target identifier
    fn name(&self) -> &str;
}
```

### Implementations

#### OpenAI

```rust
pub struct OpenAITarget {
    client: reqwest::Client,
    model: String,
    api_key: String,
}

impl OpenAITarget {
    pub fn new(api_key: &str, model: &str) -> Self;
    pub fn from_env(model: &str) -> Result<Self, TargetError>;
}
```

#### Anthropic

```rust
pub struct AnthropicTarget {
    client: reqwest::Client,
    model: String,
    api_key: String,
}
```

#### Gemini

```rust
pub struct GeminiTarget {
    client: reqwest::Client,
    model: String,
    api_key: String,
}
```

#### Mock

For testing without API calls:

```rust
pub struct MockTarget {
    responses: Vec<String>,
}

impl MockTarget {
    pub fn always_refuse() -> Self;
    pub fn always_comply() -> Self;
    pub fn with_responses(responses: Vec<String>) -> Self;
}
```

## Judge System

Evaluates attack success:

```rust
#[async_trait]
pub trait Judge: Send + Sync {
    async fn evaluate(
        &self,
        prompt: &str,
        response: &str
    ) -> Result<JudgeResult, TargetError>;
}
```

### Judge Result

```rust
pub struct JudgeResult {
    pub score: f64,           // 0.0 (safe) to 1.0 (violation)
    pub refused: bool,        // Did target refuse?
    pub harmful_content: bool, // Contains harmful info?
    pub relevant: bool,       // Addresses the request?
    pub explanation: String,  // Brief reasoning
}
```

### Heuristic Judge

Fast keyword-based evaluation:

```rust
pub struct HeuristicJudge;

impl HeuristicJudge {
    // Checks for refusal phrases:
    // "I cannot", "I can't", "I apologize", etc.

    // Checks for compliance phrases:
    // "Sure,", "Here's how", "Step 1:", etc.
}
```

### LLM Judge

Uses another LLM for accurate evaluation:

```rust
pub struct LLMJudge {
    judge_model: Arc<dyn LLMTarget>,
}

impl LLMJudge {
    pub fn new(judge_model: Arc<dyn LLMTarget>) -> Self;

    // Prompts the judge model with:
    // - The attack prompt
    // - The target's response
    // - Evaluation criteria
}
```

## Fitness Evaluator

### Simple Fitness

Fast heuristic-based:

```rust
pub struct JailbreakFitness {
    target: Arc<dyn LLMTarget>,
}
```

### Judged Fitness

With LLM judge and response storage:

```rust
pub struct JudgedJailbreakFitness {
    target: Arc<dyn LLMTarget>,
    judge: Arc<dyn Judge>,
    records: Mutex<HashMap<String, EvaluationRecord>>,
}

impl JudgedJailbreakFitness {
    pub fn new(target: Arc<dyn LLMTarget>, mode: JudgeMode) -> Self;
    pub fn get_records(&self) -> Vec<EvaluationRecord>;
    pub fn get_top_records(&self, n: usize) -> Vec<EvaluationRecord>;
}
```

### Evaluation Record

Stores full evaluation data:

```rust
pub struct EvaluationRecord {
    pub prompt: String,
    pub response: String,
    pub judge_result: Option<JudgeResult>,
    pub fitness: f64,
}
```

## Behavior Characterization

For quality-diversity:

```rust
impl LLMAttackGenome {
    fn characterize_behavior(&self) -> BehaviorDescriptor {
        BehaviorDescriptor::new(vec![
            self.primary_strategy.to_behavior_value(),
            self.encoding.to_behavior_value(),
            self.to_prompt().len() as f64,
        ])
    }
}
```
