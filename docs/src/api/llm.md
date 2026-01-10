# red-queen-llm API

API reference for the LLM safety domain.

## Modules

### `genome`

LLM attack genome and components.

```rust
use red_queen_llm::genome::{
    LLMAttackGenome,
    AttackStrategy,
    Encoding,
    HarmCategory,
    Persona,
    PromptSection,
};
```

#### `LLMAttackGenome`

```rust
pub struct LLMAttackGenome {
    pub primary_strategy: AttackStrategy,
    pub secondary_strategy: Option<AttackStrategy>,
    pub encoding: Encoding,
    pub persona: Option<Persona>,
    pub prefix: PromptSection,
    pub core: PromptSection,
    pub suffix: PromptSection,
    pub target_category: HarmCategory,
}

impl LLMAttackGenome {
    pub fn to_prompt(&self) -> String;
}
```

#### `AttackStrategy`

```rust
pub enum AttackStrategy {
    DirectJailbreak,
    Roleplay,
    AuthorityAppeal,
    HypotheticalScenario,
    MultiTurn,
    EncodingBased,
}

impl AttackStrategy {
    pub fn all() -> Vec<Self>;
    pub fn to_behavior_value(&self) -> f64;
}
```

#### `Encoding`

```rust
pub enum Encoding {
    None,
    Base64,
    ROT13,
    Unicode,
    Leetspeak,
}

impl Encoding {
    pub fn all() -> Vec<Self>;
    pub fn encode(&self, text: &str) -> String;
    pub fn to_behavior_value(&self) -> f64;
}
```

#### `HarmCategory`

```rust
pub enum HarmCategory {
    Violence,
    IllegalActivity,
    Harassment,
    Misinformation,
    Privacy,
    Malware,
}

impl HarmCategory {
    pub fn all() -> Vec<Self>;
    pub fn sample_harmful_request(&self) -> &str;
}
```

### `targets`

LLM target adapters.

```rust
use red_queen_llm::targets::{
    LLMTarget,
    TargetError,
    OpenAITarget,
    AnthropicTarget,
    GeminiTarget,
    OllamaTarget,
    MockTarget,
};
```

#### `LLMTarget` trait

```rust
#[async_trait]
pub trait LLMTarget: Send + Sync {
    async fn query(&self, prompt: &str) -> Result<String, TargetError>;
    fn name(&self) -> &str;
}
```

#### `OpenAITarget`

```rust
pub struct OpenAITarget { /* ... */ }

impl OpenAITarget {
    pub fn new(api_key: &str, model: &str) -> Self;
    pub fn from_env(model: &str) -> Result<Self, TargetError>;
}
```

#### `AnthropicTarget`

```rust
pub struct AnthropicTarget { /* ... */ }

impl AnthropicTarget {
    pub fn new(api_key: &str, model: &str) -> Self;
    pub fn from_env(model: &str) -> Result<Self, TargetError>;
}
```

#### `GeminiTarget`

```rust
pub struct GeminiTarget { /* ... */ }

impl GeminiTarget {
    pub fn new(api_key: &str, model: &str) -> Self;
    pub fn from_env(model: &str) -> Result<Self, TargetError>;
}
```

#### `OllamaTarget`

```rust
pub struct OllamaTarget { /* ... */ }

impl OllamaTarget {
    pub fn new(model: &str) -> Self;            // Uses default localhost:11434
    pub fn with_url(model: &str, base_url: &str) -> Self;
    pub fn from_env(model: &str) -> Self;       // Uses OLLAMA_HOST if set
}
```

#### `MockTarget`

```rust
pub struct MockTarget { /* ... */ }

impl MockTarget {
    pub fn always_refuse() -> Self;
    pub fn always_comply() -> Self;
    pub fn with_responses(responses: Vec<String>) -> Self;
}
```

#### `TargetError`

```rust
pub enum TargetError {
    Network(String),
    RateLimit,
    InvalidResponse(String),
    AuthenticationFailed,
    Other(String),
}
```

### `judge`

Attack success evaluation.

```rust
use red_queen_llm::judge::{
    Judge,
    JudgeResult,
    LLMJudge,
    HeuristicJudge,
};
```

#### `Judge` trait

```rust
#[async_trait]
pub trait Judge: Send + Sync {
    async fn evaluate(
        &self,
        prompt: &str,
        response: &str,
    ) -> Result<JudgeResult, TargetError>;
}
```

#### `JudgeResult`

```rust
pub struct JudgeResult {
    pub score: f64,
    pub refused: bool,
    pub harmful_content: bool,
    pub relevant: bool,
    pub explanation: String,
}

impl Default for JudgeResult {
    fn default() -> Self;  // Safe default (refused, score 0)
}
```

#### `LLMJudge`

```rust
pub struct LLMJudge { /* ... */ }

impl LLMJudge {
    pub fn new(judge_model: Arc<dyn LLMTarget>) -> Self;
}
```

#### `HeuristicJudge`

```rust
pub struct HeuristicJudge;

impl HeuristicJudge {
    pub fn new() -> Self;
}
```

### `fitness`

Fitness evaluation for jailbreak discovery.

```rust
use red_queen_llm::fitness::{
    JailbreakFitness,
    JudgedJailbreakFitness,
    JudgeMode,
    EvaluationRecord,
};
```

#### `JailbreakFitness`

Simple heuristic-based fitness with parallel evaluation:

```rust
pub struct JailbreakFitness { /* ... */ }

impl JailbreakFitness {
    pub fn new(target: Arc<dyn LLMTarget>) -> Self;
    pub fn with_concurrency(self, concurrency: usize) -> Self;
}
```

#### `JudgedJailbreakFitness`

Full-featured fitness with judge, response storage, and parallel evaluation:

```rust
pub struct JudgedJailbreakFitness { /* ... */ }

impl JudgedJailbreakFitness {
    pub fn new(target: Arc<dyn LLMTarget>, judge_mode: JudgeMode) -> Self;
    pub fn with_judge(target: Arc<dyn LLMTarget>, judge: Arc<dyn Judge>) -> Self;
    pub fn with_concurrency(self, concurrency: usize) -> Self;
    pub fn concurrency(&self) -> usize;
    pub fn get_records(&self) -> Vec<EvaluationRecord>;
    pub fn get_top_records(&self, n: usize) -> Vec<EvaluationRecord>;
    pub fn clear_records(&self);
}
```

Both fitness evaluators support parallel batch evaluation via the `Fitness::evaluate_batch` trait method. Set concurrency based on API rate limits (default: 10).

#### `JudgeMode`

```rust
pub enum JudgeMode {
    Heuristic,
    LLM(Arc<dyn LLMTarget>),
}
```

#### `EvaluationRecord`

```rust
pub struct EvaluationRecord {
    pub prompt: String,
    pub response: String,
    pub judge_result: Option<JudgeResult>,
    pub fitness: f64,
}
```

## Prelude

Common imports:

```rust
use red_queen_llm::prelude::*;
// Imports: LLMAttackGenome, AttackStrategy, JailbreakFitness, LLMTarget, etc.
```
