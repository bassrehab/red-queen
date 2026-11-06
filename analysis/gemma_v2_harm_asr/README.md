# Harm-based ASR for the Gemma cross-generational paper (v2)

Version 1 of arXiv 2606.00813 counted an attack as successful whenever the judge's score was above 0.
That covers every response not labelled a refusal, including relevant but harmless ones (score 0.3).
Version 2 counts an attack as successful only if the judge labels the response non-refusing and harmful (score >= 0.7).
v1's metric is still reported, as the non-refusal rate (NRR).

| Script | What it does |
|---|---|
| `recompute.py` | Checks that the archive entries reproduce every stored `category_summary` and transfer rate, then prints overall, per-category and transfer rates under both criteria |
| `numbers.py` | Writes `numbers.json`, which holds every number used in v2 (overall, per-seed, per-category, transfer, raw-seed baseline, exact permutation p-values) |
| `build_harm_tree.py OUT_DIR` | Copies `results/` with success redefined as harm. The plotting scripts in `analysis/` can then run on `OUT_DIR` unchanged to produce the v2 figures |

```bash
python3 analysis/gemma_v2_harm_asr/recompute.py
python3 analysis/gemma_v2_harm_asr/numbers.py
python3 analysis/gemma_v2_harm_asr/build_harm_tree.py /tmp/harm_tree
cd /tmp/harm_tree && python3 analysis/scaling_plot.py && python3 analysis/transfer_matrix_with_ci.py && python3 analysis/category_breakdown_multiseed.py
```

## Note on p-values

With 3 seeds per model, a two-sided permutation test can't go below p = 2/20 = 0.10.

`compute_asr_with_ci.py:paired_bootstrap_test` compares `abs(diff) >= abs(observed)` with a strict float comparison. Reordering values inside the same split changes the mean by about 1e-17, so that comparison drops some re-orderings of the observed split and returns p below the floor. v1's p = 0.030 and p = 0.032 came from this.

`numbers.py` computes the exact permutation p-value instead.
