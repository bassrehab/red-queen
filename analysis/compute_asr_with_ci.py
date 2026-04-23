#!/usr/bin/env python3
"""
Phase 2A: Compute ASR with bootstrap confidence intervals.

For each model, loads 3 seed archives, computes per-category and overall ASR
with 95% bootstrap CIs (10,000 resamples). Produces CSV and LaTeX table.

Usage:
    python analysis/compute_asr_with_ci.py
"""

import json
import os
import csv
import numpy as np
from collections import defaultdict

MODELS = [
    ("gemma1_7b", "Gemma 1 (7B)"),
    ("gemma2_9b", "Gemma 2 (9B)"),
    ("gemma3_12b", "Gemma 3 (12B)"),
    ("gemma4_31b", "Gemma 4 (31B)"),
]
SEEDS = [42, 1337, 2718]
RESULTS_DIR = "results"
N_BOOTSTRAP = 10000
RNG = np.random.RandomState(42)


def load_category_data():
    """Load per-category success rates for each (model, seed)."""
    data = {}
    for model_key, _ in MODELS:
        data[model_key] = {}
        for seed in SEEDS:
            path = os.path.join(RESULTS_DIR, model_key, f"seed_{seed}", "results.json")
            if not os.path.exists(path):
                continue
            with open(path) as f:
                d = json.load(f)
            cs = d.get("category_summary", {})
            data[model_key][seed] = cs
    return data


def bootstrap_ci(values, n_bootstrap=N_BOOTSTRAP, ci=0.95):
    """Compute bootstrap confidence interval for the mean."""
    values = np.array(values)
    if len(values) < 2:
        return values.mean(), 0.0, values.mean(), values.mean()

    boot_means = np.array([
        RNG.choice(values, size=len(values), replace=True).mean()
        for _ in range(n_bootstrap)
    ])
    mean = values.mean()
    std = values.std(ddof=1)
    alpha = (1 - ci) / 2
    ci_low = np.percentile(boot_means, alpha * 100)
    ci_high = np.percentile(boot_means, (1 - alpha) * 100)
    return mean, std, ci_low, ci_high


def paired_bootstrap_test(values_a, values_b, n_bootstrap=N_BOOTSTRAP):
    """Paired bootstrap test: is mean(A) != mean(B)?"""
    values_a = np.array(values_a)
    values_b = np.array(values_b)
    observed_diff = values_a.mean() - values_b.mean()
    combined = np.concatenate([values_a, values_b])
    n = len(values_a)
    count = 0
    for _ in range(n_bootstrap):
        perm = RNG.permutation(combined)
        diff = perm[:n].mean() - perm[n:].mean()
        if abs(diff) >= abs(observed_diff):
            count += 1
    return count / n_bootstrap


def main():
    data = load_category_data()

    # Gather all categories
    all_cats = set()
    for model_key, _ in MODELS:
        for seed_data in data[model_key].values():
            all_cats.update(seed_data.keys())
    categories = sorted(all_cats)

    # Compute per-model, per-category stats
    results = []
    for model_key, model_label in MODELS:
        for cat in categories + ["overall"]:
            rates = []
            for seed in SEEDS:
                if seed not in data[model_key]:
                    continue
                cs = data[model_key][seed]
                if cat == "overall":
                    total = sum(v["total_attacks"] for v in cs.values())
                    success = sum(v["successful_attacks"] for v in cs.values())
                    rates.append(success / total if total > 0 else 0.0)
                elif cat in cs:
                    rates.append(cs[cat]["success_rate"])

            if not rates:
                continue

            mean, std, ci_low, ci_high = bootstrap_ci(rates)
            results.append({
                "model": model_label,
                "model_key": model_key,
                "category": cat,
                "mean": mean,
                "std": std,
                "ci_low": ci_low,
                "ci_high": ci_high,
                "n_seeds": len(rates),
            })

    # Write CSV
    csv_path = "analysis/output/asr_summary.csv"
    with open(csv_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=[
            "model", "category", "mean", "std", "ci_low", "ci_high", "n_seeds"
        ])
        writer.writeheader()
        for r in results:
            writer.writerow({k: r[k] for k in writer.fieldnames})
    print(f"CSV saved: {csv_path}")

    # Write LaTeX table
    tex_path = "analysis/output/asr_summary.tex"
    cat_short = {
        "chemical_biological": "Chem/Bio",
        "copyright": "Copyright",
        "cybercrime_intrusion": "Cybercrime",
        "harassment_bullying": "Harassment",
        "harmful": "Harmful",
        "illegal": "Illegal",
        "misinformation_disinformation": "Misinfo",
        "overall": "\\textbf{Overall}",
    }

    # Find lowest ASR model per category for bolding
    cat_best = {}
    for cat in categories + ["overall"]:
        cat_results = [r for r in results if r["category"] == cat]
        if cat_results:
            cat_best[cat] = min(cat_results, key=lambda x: x["mean"])["model_key"]

    with open(tex_path, "w") as f:
        f.write("\\begin{table}[t]\n\\centering\n")
        f.write("\\caption{Attack success rate (\\%) per HarmBench category across Gemma\n")
        f.write("generations. Mean $\\pm$ std over 3 seeds. 95\\% bootstrap CIs in brackets.\n")
        f.write("\\textbf{Bold} indicates lowest (safest) ASR per category.}\n")
        f.write("\\label{tab:asr}\n")
        f.write("\\begin{tabular}{l" + "c" * len(MODELS) + "}\n\\toprule\n")
        f.write("\\textbf{Category}")
        for _, label in MODELS:
            f.write(f" & \\textbf{{{label}}}")
        f.write(" \\\\\n\\midrule\n")

        for cat in categories + ["overall"]:
            if cat == "overall":
                f.write("\\midrule\n")
            f.write(cat_short.get(cat, cat))
            for model_key, _ in MODELS:
                r = next((x for x in results if x["model_key"] == model_key and x["category"] == cat), None)
                if r:
                    val = f"{r['mean']*100:.1f} $\\pm$ {r['std']*100:.1f}"
                    if cat_best.get(cat) == model_key:
                        val = f"\\textbf{{{val}}}"
                    f.write(f" & {val}")
                else:
                    f.write(" & ---")
            f.write(" \\\\\n")

        f.write("\\bottomrule\n\\end{tabular}\n\\end{table}\n")
    print(f"LaTeX saved: {tex_path}")

    # Print diagnostic
    print("\nDiagnostic — std/mean ratio for overall ASR:")
    for model_key, model_label in MODELS:
        r = next((x for x in results if x["model_key"] == model_key and x["category"] == "overall"), None)
        if r and r["mean"] > 0:
            ratio = r["std"] / r["mean"]
            flag = " ⚠️  HIGH VARIANCE" if ratio > 0.15 else ""
            print(f"  {model_label}: std/mean = {ratio:.2f} (std={r['std']:.3f}, mean={r['mean']:.3f}){flag}")

    # Paired significance tests (Gemma 3 vs Gemma 2, Gemma 3 vs Gemma 4)
    print("\nPaired bootstrap tests (overall ASR):")
    for a_key, a_label in MODELS:
        for b_key, b_label in MODELS:
            if a_key >= b_key:
                continue
            rates_a = []
            rates_b = []
            for seed in SEEDS:
                if seed in data[a_key] and seed in data[b_key]:
                    cs_a = data[a_key][seed]
                    cs_b = data[b_key][seed]
                    total_a = sum(v["total_attacks"] for v in cs_a.values())
                    succ_a = sum(v["successful_attacks"] for v in cs_a.values())
                    total_b = sum(v["total_attacks"] for v in cs_b.values())
                    succ_b = sum(v["successful_attacks"] for v in cs_b.values())
                    rates_a.append(succ_a / total_a if total_a > 0 else 0)
                    rates_b.append(succ_b / total_b if total_b > 0 else 0)
            if len(rates_a) >= 2:
                p = paired_bootstrap_test(rates_a, rates_b)
                sig = "***" if p < 0.001 else "**" if p < 0.01 else "*" if p < 0.05 else "ns"
                diff = np.mean(rates_a) - np.mean(rates_b)
                print(f"  {a_label} vs {b_label}: diff={diff:+.1%}, p={p:.4f} {sig}")


if __name__ == "__main__":
    main()
