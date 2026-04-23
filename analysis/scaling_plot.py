#!/usr/bin/env python3
"""
Phase 2C: Scaling plot — honest, no regression fit.

Shows ASR vs model size with error bars. Highlights the non-monotonic
pattern with Gemma 3 as an outlier. No curve fitting on 4 points.
"""

import json
import os
import numpy as np
import matplotlib
import matplotlib.pyplot as plt

matplotlib.rcParams.update({
    "font.family": "serif",
    "font.size": 10,
    "axes.titlesize": 12,
    "figure.dpi": 150,
})

MODELS = [
    ("gemma1_7b", "Gemma 1", 7e9),
    ("gemma2_9b", "Gemma 2", 9e9),
    ("gemma3_12b", "Gemma 3", 12e9),
    ("gemma4_31b", "Gemma 4", 31e9),
]
SEEDS = [42, 1337, 2718]
RESULTS_DIR = "results"
RNG = np.random.RandomState(42)


def load_asr():
    results = []
    for model_key, label, params in MODELS:
        asrs = []
        for seed in SEEDS:
            path = os.path.join(RESULTS_DIR, model_key, f"seed_{seed}", "results.json")
            if not os.path.exists(path):
                continue
            with open(path) as f:
                d = json.load(f)
            cs = d.get("category_summary", {})
            total = sum(v["total_attacks"] for v in cs.values())
            success = sum(v["successful_attacks"] for v in cs.values())
            asrs.append(success / total if total > 0 else 0)
        if asrs:
            results.append({
                "key": model_key,
                "label": label,
                "params": params,
                "asrs": asrs,
                "mean": np.mean(asrs),
                "std": np.std(asrs, ddof=1) if len(asrs) > 1 else 0,
            })
    return results


def main():
    results = load_asr()
    if len(results) < 2:
        print("Need at least 2 models")
        return

    params = np.array([r["params"] for r in results])
    means = np.array([r["mean"] for r in results])
    stds = np.array([r["std"] for r in results])
    labels = [r["label"] for r in results]
    robustness = 1 - means
    rob_stds = stds  # same magnitude

    fig, axes = plt.subplots(1, 2, figsize=(11, 4.5))

    # Identify Gemma 3 as outlier
    colors = ["#2ca02c" if l != "Gemma 3" else "#d62728" for l in labels]

    # Left: ASR vs params
    ax = axes[0]
    ax.errorbar(params / 1e9, means, yerr=stds, fmt="none", ecolor="gray",
                capsize=4, capthick=1, zorder=3)
    for i, (p, m, l, c) in enumerate(zip(params, means, labels, colors)):
        ax.scatter(p / 1e9, m, s=100, c=c, zorder=5, edgecolors="black", linewidth=0.5)
        ax.annotate(l, (p / 1e9, m), textcoords="offset points",
                    xytext=(10, 8), fontsize=9)

    # Shaded "expected" band (Gemma 2 to Gemma 4 range, excluding Gemma 3)
    non_g3 = [r for r in results if r["label"] != "Gemma 3"]
    if len(non_g3) >= 2:
        expected_low = min(r["mean"] for r in non_g3)
        expected_high = max(r["mean"] for r in non_g3)
        ax.axhspan(expected_low - 0.05, expected_high + 0.05, alpha=0.08,
                   color="green", label="Expected range\n(excl. Gemma 3)")

    ax.set_xscale("log")
    ax.set_xlabel("Parameters (billions)")
    ax.set_ylabel("Attack Success Rate")
    ax.set_title("Attack Success Rate vs. Model Size")
    ax.set_ylim(-0.05, 1.05)
    ax.grid(True, alpha=0.3)
    ax.legend(fontsize=7, loc="upper right")

    # Right: Robustness vs params
    ax = axes[1]
    ax.errorbar(params / 1e9, robustness, yerr=rob_stds, fmt="none",
                ecolor="gray", capsize=4, capthick=1, zorder=3)
    for i, (p, r, l, c) in enumerate(zip(params, robustness, labels, colors)):
        ax.scatter(p / 1e9, r, s=100, c=c, zorder=5, edgecolors="black", linewidth=0.5)
        ax.annotate(l, (p / 1e9, r), textcoords="offset points",
                    xytext=(10, 8), fontsize=9)

    ax.set_xscale("log")
    ax.set_xlabel("Parameters (billions)")
    ax.set_ylabel("Robustness (1 - ASR)")
    ax.set_title("Adversarial Robustness vs. Model Size")
    ax.set_ylim(-0.05, 1.05)
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    out = "figures/scaling_curve.pdf"
    os.makedirs(os.path.dirname(out), exist_ok=True)
    fig.savefig(out, bbox_inches="tight")
    print(f"Saved: {out}")
    plt.close()

    # Print summary
    print("\nScaling summary (mean ± std across 3 seeds):")
    print(f"{'Model':<12} {'Params':>8} {'ASR':>12} {'Robust':>12}")
    for r in results:
        print(f"{r['label']:<12} {r['params']/1e9:>7.0f}B {r['mean']:>6.1%} ± {r['std']:>4.1%} {1-r['mean']:>6.1%} ± {r['std']:>4.1%}")


if __name__ == "__main__":
    main()
