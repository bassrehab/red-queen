//! # Red Queen LLM
//!
//! LLM safety domain for adversarial testing.
//!
//! This crate provides:
//! - Semantic genome for LLM attacks
//! - Jailbreak fitness evaluation
//! - LLM Judge for accurate attack success detection
//! - Target adapters (OpenAI, Anthropic, etc.)
//! - Behavior characterization for QD

pub mod genome;
pub mod fitness;
pub mod judge;
pub mod targets;
pub mod mutations;
pub mod seeds;

pub mod prelude {
    pub use crate::genome::{LLMAttackGenome, AttackStrategy, HarmCategory};
    pub use crate::fitness::{JailbreakFitness, JudgedJailbreakFitness};
    pub use crate::judge::{Judge, JudgeResult, LLMJudge, HeuristicJudge};
    pub use crate::targets::{LLMTarget, OpenAITarget, AnthropicTarget, GeminiTarget, OllamaTarget};
    pub use crate::seeds::{SeedBehavior, SeedPool};
}
