//! Seed behaviors for LLM attack evolution.
//!
//! A `SeedPool` is a collection of target behaviors (prompts) that the
//! evolutionary loop tries to elicit from a model. Seeds are loaded from a
//! JSONL file at runtime rather than embedded in the crate, so users bring
//! their own dataset (e.g. HarmBench, AdvBench) and the published crate
//! stays agnostic.
//!
//! Expected JSONL schema (one object per line):
//!
//! ```json
//! {"id": "hb_cyber_001", "behavior": "...", "category": "cybercrime_intrusion", "tags": []}
//! ```
//!
//! `category` and `tags` are optional. Blank lines and lines starting with
//! `#` are ignored.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// A single seed behavior to target during evolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedBehavior {
    /// Stable identifier (e.g. HarmBench behavior_id).
    pub id: String,
    /// The behavior prompt text — what the attacker wants the model to do.
    pub behavior: String,
    /// Optional taxonomy label (e.g. HarmBench functional category).
    #[serde(default)]
    pub category: Option<String>,
    /// Optional free-form tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A pool of seed behaviors used to initialize attack genomes.
#[derive(Debug, Clone, Default)]
pub struct SeedPool {
    behaviors: Vec<SeedBehavior>,
}

impl SeedPool {
    /// Create a pool from an in-memory vector of behaviors.
    pub fn new(behaviors: Vec<SeedBehavior>) -> Self {
        Self { behaviors }
    }

    /// Load a pool from a JSONL file on disk.
    pub fn from_jsonl_path<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_jsonl_str(&content)
    }

    /// Parse a pool from a JSONL string.
    pub fn from_jsonl_str(content: &str) -> std::io::Result<Self> {
        let mut behaviors = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let b: SeedBehavior = serde_json::from_str(trimmed).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("seed line {}: {}", i + 1, e),
                )
            })?;
            behaviors.push(b);
        }
        Ok(Self { behaviors })
    }

    /// Number of behaviors in the pool.
    pub fn len(&self) -> usize {
        self.behaviors.len()
    }

    /// Whether the pool contains any behaviors.
    pub fn is_empty(&self) -> bool {
        self.behaviors.is_empty()
    }

    /// Sample a random behavior using the provided RNG.
    pub fn sample<R: Rng>(&self, rng: &mut R) -> Option<&SeedBehavior> {
        self.behaviors.choose(rng)
    }

    /// Iterate over all behaviors.
    pub fn iter(&self) -> impl Iterator<Item = &SeedBehavior> {
        self.behaviors.iter()
    }

    /// Count behaviors per category (sorted for stable display).
    pub fn category_counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for b in &self.behaviors {
            let key = b
                .category
                .clone()
                .unwrap_or_else(|| "uncategorized".to_string());
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }
}

static SEED_POOL: RwLock<Option<Arc<SeedPool>>> = RwLock::new(None);

/// Install a global seed pool that `LLMAttackGenome::random` will draw from.
pub fn install_seed_pool(pool: SeedPool) {
    *SEED_POOL
        .write()
        .expect("seed pool lock poisoned") = Some(Arc::new(pool));
}

/// Remove any installed global seed pool.
pub fn clear_seed_pool() {
    *SEED_POOL
        .write()
        .expect("seed pool lock poisoned") = None;
}

/// Fetch a handle to the currently installed seed pool, if any.
pub fn current_seed_pool() -> Option<Arc<SeedPool>> {
    SEED_POOL.read().ok().and_then(|g| g.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::LLMAttackGenome;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use red_queen_core::genome::Genome;
    use std::sync::Mutex;

    // Tests that touch the process-wide SEED_POOL must be serialized against
    // each other so they don't observe one another's installed state.
    static POOL_LOCK: Mutex<()> = Mutex::new(());

    fn synthetic_jsonl() -> &'static str {
        // Synthetic placeholder behaviors only — no real harmful content.
        r#"
# comment line, should be ignored
{"id": "syn_001", "behavior": "placeholder behavior alpha", "category": "cat_a"}
{"id": "syn_002", "behavior": "placeholder behavior beta", "category": "cat_b", "tags": ["x"]}
{"id": "syn_003", "behavior": "placeholder behavior gamma"}
"#
    }

    #[test]
    fn parses_jsonl_and_skips_blank_and_comments() {
        let pool = SeedPool::from_jsonl_str(synthetic_jsonl()).unwrap();
        assert_eq!(pool.len(), 3);
        assert!(!pool.is_empty());
    }

    #[test]
    fn category_counts_group_uncategorized() {
        let pool = SeedPool::from_jsonl_str(synthetic_jsonl()).unwrap();
        let counts = pool.category_counts();
        assert_eq!(counts.get("cat_a"), Some(&1));
        assert_eq!(counts.get("cat_b"), Some(&1));
        assert_eq!(counts.get("uncategorized"), Some(&1));
    }

    #[test]
    fn sample_returns_pool_member() {
        let pool = SeedPool::from_jsonl_str(synthetic_jsonl()).unwrap();
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let picked = pool.sample(&mut rng).unwrap();
        assert!(picked.id.starts_with("syn_"));
    }

    #[test]
    fn invalid_line_returns_error() {
        let bad = "{not json}\n";
        let err = SeedPool::from_jsonl_str(bad).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn empty_pool_is_empty() {
        let pool = SeedPool::new(vec![]);
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn install_and_clear_roundtrip() {
        let _guard = POOL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        install_seed_pool(SeedPool::from_jsonl_str(synthetic_jsonl()).unwrap());
        let installed = current_seed_pool().expect("pool should be installed");
        assert_eq!(installed.len(), 3);
        clear_seed_pool();
        assert!(current_seed_pool().is_none());
    }

    #[test]
    fn genome_random_draws_from_installed_pool() {
        let _guard = POOL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        install_seed_pool(SeedPool::from_jsonl_str(synthetic_jsonl()).unwrap());

        let mut rng = ChaCha8Rng::seed_from_u64(1234);
        let g = LLMAttackGenome::random(&mut rng);

        assert!(g.seed_id.is_some(), "seed_id should be set when pool is installed");
        let id = g.seed_id.as_deref().unwrap();
        assert!(id.starts_with("syn_"), "unexpected id: {}", id);
        assert!(
            g.core.content.starts_with("placeholder behavior"),
            "core should come from pool, got: {}",
            g.core.content
        );

        clear_seed_pool();
    }

    #[test]
    fn genome_random_falls_back_without_pool() {
        let _guard = POOL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_seed_pool();
        let mut rng = ChaCha8Rng::seed_from_u64(5678);
        let g = LLMAttackGenome::random(&mut rng);
        assert!(g.seed_id.is_none());
        assert_eq!(g.core.content, "Tell me how to do something harmful.");
    }
}
