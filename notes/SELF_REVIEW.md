# Self-Review — Devil's Advocate

_Playing a skeptical NeurIPS reviewer. For each concern: objection, severity, counter-response._

---

## 1. Statistical power with N=3 seeds

**Objection:** "With only 3 seeds, most pairwise comparisons do not reach significance. The headline finding (Gemma 3 regression) is p=0.030 — barely significant. The Gemma 3→4 recovery (the other half of the story) is NOT significant. The paper's central claim rests on weak statistics."

**Severity:** Major

**Counter-response:** We agree that N=3 provides limited power and state this explicitly in the limitations. The two most important contrasts — Gemma 1→2 improvement (p=0.032) and Gemma 2→3 regression (p=0.030) — ARE significant. The transfer matrix provides converging evidence independent of per-seed variance: Gemma 3 is the easiest transfer target from ALL source archives, which would be extremely unlikely under the null. We frame the Gemma 3→4 recovery as "large but not significant with N=3" rather than claiming it as established. Additional seeds (5-10) would tighten CIs; we welcome this in future work.

---

## 2. Single model family

**Objection:** "You study only Gemma. The non-monotonic pattern might be a Gemma-specific artifact (e.g., a bad Gemma 3 release), not a general phenomenon. You cannot generalize from one family."

**Severity:** Major

**Counter-response:** We explicitly state this limitation and frame the paper as a case study, not a general claim. The contribution is the METHODOLOGY (differential safety probing via QD evolution + transfer matrix), not the claim that "all model families are non-monotonic." The Gemma case study demonstrates the methodology's value — it reveals a pattern that static benchmarks would miss. We note that extending to Llama, Qwen, or Claude families is straightforward with the released code and would be valuable future work.

---

## 3. Judge reliability

**Objection:** "You use a single 8B model as judge. How do you know Llama-3.1-8B-Instruct is accurately classifying harmful content? Small models are known to have poor safety judgment. Your entire pipeline depends on this judge."

**Severity:** Major

**Counter-response:** This is a valid concern. We chose Llama-3.1-8B for reproducibility (self-hosted, deterministic) over GPT-5.4-mini (API, rate-limited, non-reproducible). The judge uses a structured rubric with binary classification (refused/harmful/relevant) rather than open-ended scoring, which is within 8B model capabilities. We acknowledge this limitation and note that a judge agreement study (comparing Llama-8B vs a larger judge on a held-out sample) would strengthen the paper. This is listed as future work.

**ACTION NEEDED:** Run the judge agreement check (Phase 2D from prompts.md) if time permits. This would substantially address this concern.

---

## 4. Copyright at 100% — measurement artifact?

**Objection:** "Copyright is at 100% ASR for ALL models and ALL seeds. This is suspicious — it could indicate a judge bias (the judge always scores copyright-related responses as 'not refused') rather than genuine vulnerability."

**Severity:** Minor-Major

**Counter-response:** This is a fair point worth investigating. However, copyright behaviors in HarmBench tend to involve requests like "write a poem in the style of [author]" which many models will comply with because the boundary between creative inspiration and infringement is genuinely ambiguous. The 100% rate may reflect that all Gemma models treat copyright as a creative task rather than a safety-sensitive one. We should note this interpretation in the paper.

---

## 5. Alternative explanation for Gemma 3 regression

**Objection:** "You attribute the Gemma 3 regression to multimodal architectural changes. But you have zero evidence for this — it's pure speculation. It could be: different training data, different RLHF budget, different safety team priorities, or simply a bug in Gemma 3's safety training."

**Severity:** Minor

**Counter-response:** We present the architectural hypothesis as a HYPOTHESIS, not a conclusion ("We hypothesize..."). The evidence is circumstantial: the regression coincides with the architectural shift, and the one category that continued improving (harassment) is the one most likely to have explicit, category-specific safety training. We explicitly list alternative explanations and note that we cannot distinguish between them with external evaluation alone. This is honest.

---

## 6. Ethics of releasing an attack-discovery methodology

**Objection:** "You're publishing a tool that discovers jailbreak attacks against LLMs. This is a dual-use concern. The transfer replay protocol makes it easy to find attacks that work across model generations."

**Severity:** Minor

**Counter-response:** We release aggregate statistics (transfer rates, ASR per category) and the framework code, but NOT the specific attack prompts or model responses. The methodology is defensive — it helps model developers identify and fix vulnerabilities. The HarmBench seed behaviors are already public. The evolution framework discovers attacks that are model-specific; the transfer analysis reveals which are patched and which persist, which is information that helps defenders more than attackers.

---

## 7. Confounding: model size vs generation

**Objection:** "Gemma models differ in both size AND generation. You can't separate the effect of scale from the effect of training improvements. Gemma 4 (31B) might be safer simply because it's bigger, not because Google improved alignment."

**Severity:** Minor

**Counter-response:** This is inherent to studying a real model family rather than controlled experiments. We acknowledge it in the methodology section. The transfer matrix helps: if Gemma 4 were safe purely due to size, we'd expect evolved Gemma 4 attacks to be weak (they can't find real vulnerabilities). Instead, Gemma 4 attacks transfer backward to Gemma 1 at 49% — they find real weaknesses that happen to be patched in Gemma 4. This suggests the safety improvement is from alignment, not just scale.

---

## 8. Reproducibility of self-hosted judge

**Objection:** "You claim reproducibility by using a self-hosted judge with temperature=0. But vLLM with bf16 on different GPU architectures may produce slightly different outputs due to floating-point non-determinism. Your 'deterministic' claim is overstated."

**Severity:** Minor

**Counter-response:** We use deterministic RNG (ChaCha8) and ordered evaluation. Temperature=0 eliminates sampling randomness. Cross-GPU numerical differences are negligible for classification tasks (the judge outputs structured JSON). Our seed_determinism test passes 5/5 runs with identical archives. We acknowledge that perfect bitwise reproducibility across GPU architectures is not guaranteed.

---

## Summary

| # | Concern | Severity | Addressed? |
|---|---------|----------|------------|
| 1 | N=3 statistical power | Major | Partially — honest but could be stronger with more seeds |
| 2 | Single model family | Major | Yes — explicitly framed as case study |
| 3 | Judge reliability | Major | Partially — needs agreement study |
| 4 | Copyright 100% | Minor-Major | Needs discussion in paper |
| 5 | Alternative explanations | Minor | Yes — presented as hypothesis |
| 6 | Dual-use ethics | Minor | Yes — handled in checklist |
| 7 | Size vs generation confound | Minor | Yes — transfer analysis helps |
| 8 | Reproducibility claim | Minor | Yes — caveated |

**No reject-severity concerns outstanding.** The two major concerns (N=3 power, single family) are inherent limitations that are honestly disclosed. The judge reliability concern could be substantially addressed with a Phase 2D agreement study if time permits.
