//! # Red Queen LLM
//!
//! LLM safety domain for adversarial testing.
//!
//! This crate provides:
//! - Semantic genome for LLM attacks
//! - Jailbreak fitness evaluation
//! - Target adapters (OpenAI, Anthropic, etc.)
//! - Behavior characterization for QD

pub mod genome;
pub mod fitness;
pub mod targets;
pub mod mutations;

pub mod prelude {
    pub use crate::genome::{LLMAttackGenome, AttackStrategy, HarmCategory};
    pub use crate::fitness::JailbreakFitness;
    pub use crate::targets::{LLMTarget, OpenAITarget};
}
