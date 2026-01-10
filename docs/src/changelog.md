# Changelog

All notable changes to Red Queen will be documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- MAP-Elites quality-diversity evolution
  - `MapElitesEvolution` engine for diverse solution discovery
  - `MapElitesArchive` with `best()`, `top_n()`, `select_random()`, `get_cell()`
  - `MapElitesConfig` and `MapElitesResult` types
  - `BehaviorDimension` for configuring archive grid
  - `ArchiveCoverage` statistics
  - `--map-elites` CLI flag to enable quality-diversity mode
  - `--me-bins` to configure bins per dimension
  - `--me-batch-size` for offspring per generation
  - Coverage statistics displayed in results
- LLM Judge system for accurate attack success evaluation
  - `LLMJudge` - Uses an LLM to evaluate responses
  - `HeuristicJudge` - Fast keyword-based fallback
  - `JudgeResult` with score, refused, harmful_content, relevant fields
- Response storage for post-evolution analysis
  - `EvaluationRecord` captures prompt, response, judge result
  - `--store-responses` CLI flag
  - Results JSON includes full response data
- `--judge <MODEL>` CLI option to specify judge model
- Blanket impl for `Fitness<G>` on references

### Changed

- `AttackResult` now includes optional `response` and `judge_result` fields
- CLI output shows judge evaluation details for top attacks
- `Archive` trait now includes `len()` and `is_empty()` methods

## [0.1.0] - 2025-12-27

### Added

- Initial release
- Core evolution engine (`red-queen-core`)
  - `Genome` trait for evolved solutions
  - `Fitness` trait for evaluation
  - `Selection` trait with `Tournament` implementation
  - `Population` management
  - `Evolution` engine with configurable parameters
  - `Archive` structure for quality-diversity (basic)
  - `BehaviorDescriptor` for QD characterization
- LLM safety domain (`red-queen-llm`)
  - `LLMAttackGenome` with strategies, encodings, personas
  - `AttackStrategy`: DirectJailbreak, Roleplay, Authority, Hypothetical, MultiTurn, Encoding
  - `Encoding`: None, Base64, ROT13, Unicode, Leetspeak
  - `HarmCategory` taxonomy
  - `JailbreakFitness` evaluator
  - Target adapters: OpenAI, Anthropic, Gemini, Mock
- CLI (`red-queen-cli`)
  - `llm jailbreak` command with full configuration
  - `analyze` command for results inspection
  - JSON output format
  - Progress bar display
- Documentation
  - mdBook-based documentation site
  - API reference
  - Architecture guides
  - Concept explanations

### Security

- API keys via environment variables only
- No credential storage
- Responsible use guidelines

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.0 | 2025-12-27 | Initial release |

## Upgrade Guide

### From Pre-Release to 0.1.0

This is the initial release. No upgrade path needed.

### Future Upgrades

Check this section for breaking changes and migration guides when upgrading between versions.
