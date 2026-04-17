#!/usr/bin/env python3
"""
Generate the robustness vs. parameter count scaling curve (Figure 3).

Reads results_gemma*.json files and plots multiple robustness metrics
against model size on a log scale. Fits power-law and log-linear models.

Usage:
    python scripts/scaling_curve.py [--experiments-dir experiments] [--output figures/scaling_curve.pdf]
"""

import argparse
import json
import os
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from scipy import optimize
import matplotlib

matplotlib.rcParams.update({
    "font.family": "serif",
    "font.size": 10,
    "axes.titlesize": 12,
    "figure.dpi": 150,
})

GEMMA_MODELS = [
    ("gemma1_7b", "Gemma 1", 7e9),
    ("gemma2_9b", "Gemma 2", 9e9),
    ("gemma3_12b", "Gemma 3", 12e9),
    ("gemma4_31b", "Gemma 4", 31e9),
]


def load_results(experiments_dir: str) -> list:
    """Load results for each Gemma model."""
    results = []
    for key, label, params in GEMMA_MODELS:
        path = os.path.join(experiments_dir, f"results_{key}.json")
        if not os.path.exists(path):
            continue
        with open(path) as f:
            data = json.load(f)

        cs = data.get("category_summary", {})
        if cs:
            total = sum(v["total_attacks"] for v in cs.values())
            success = sum(v["successful_attacks"] for v in cs.values())
            asr = success / total if total > 0 else 0.0
            avg_fitness = sum(v["avg_fitness"] for v in cs.values()) / len(cs) if cs else 0.0
            best_fitness = max(v["best_fitness"] for v in cs.values()) if cs else 0.0
        else:
            asr = 0.0
            avg_fitness = data.get("best_attack", {}).get("fitness", 0.0)
            best_fitness = avg_fitness

        results.append({
            "key": key,
            "label": label,
            "params": params,
            "asr": asr,
            "avg_fitness": avg_fitness,
            "best_fitness": best_fitness,
            "n_categories": len(cs),
        })

    return results


def fit_power_law(x, y):
    """Fit y = a * x^b, return (a, b, r_squared)."""
    try:
        log_x = np.log(x)
        log_y = np.log(np.clip(y, 1e-10, None))
        coeffs = np.polyfit(log_x, log_y, 1)
        b, log_a = coeffs
        a = np.exp(log_a)

        y_pred = a * x ** b
        ss_res = np.sum((y - y_pred) ** 2)
        ss_tot = np.sum((y - np.mean(y)) ** 2)
        r2 = 1 - ss_res / ss_tot if ss_tot > 0 else 0.0

        return a, b, r2
    except Exception:
        return None, None, 0.0


def fit_log_linear(x, y):
    """Fit y = a * log(x) + b, return (a, b, r_squared)."""
    try:
        log_x = np.log(x)
        coeffs = np.polyfit(log_x, y, 1)
        a, b = coeffs

        y_pred = a * log_x + b
        ss_res = np.sum((y - y_pred) ** 2)
        ss_tot = np.sum((y - np.mean(y)) ** 2)
        r2 = 1 - ss_res / ss_tot if ss_tot > 0 else 0.0

        return a, b, r2
    except Exception:
        return None, None, 0.0


def plot_scaling(results: list, output_path: str):
    """Plot robustness metrics vs parameter count."""
    if len(results) < 2:
        print(f"Need at least 2 models, got {len(results)}")
        return

    params = np.array([r["params"] for r in results])
    asr = np.array([r["asr"] for r in results])
    avg_fit = np.array([r["avg_fitness"] for r in results])
    labels = [r["label"] for r in results]

    # Robustness = 1 - ASR (higher = more robust)
    robustness = 1.0 - asr

    fig, axes = plt.subplots(1, 2, figsize=(11, 4.5))

    # Left: ASR vs params
    ax = axes[0]
    ax.scatter(params / 1e9, asr, s=80, c="#d62728", zorder=5, edgecolors="black", linewidth=0.5)
    for i, label in enumerate(labels):
        ax.annotate(label, (params[i] / 1e9, asr[i]),
                    textcoords="offset points", xytext=(8, 5), fontsize=8)

    # Fit
    if len(params) >= 3:
        a, b, r2 = fit_log_linear(params, asr)
        if a is not None:
            x_fit = np.linspace(params.min() * 0.8, params.max() * 1.2, 100)
            y_fit = a * np.log(x_fit) + b
            ax.plot(x_fit / 1e9, y_fit, "--", color="gray", alpha=0.7,
                    label=f"log-linear (R²={r2:.2f})")
            ax.legend(fontsize=8)

    ax.set_xscale("log")
    ax.set_xlabel("Parameters (billions)")
    ax.set_ylabel("Attack Success Rate")
    ax.set_title("Attack Success Rate vs. Model Size")
    ax.set_ylim(-0.05, 1.05)
    ax.grid(True, alpha=0.3)

    # Right: Robustness vs params
    ax = axes[1]
    ax.scatter(params / 1e9, robustness, s=80, c="#2ca02c", zorder=5,
               edgecolors="black", linewidth=0.5)
    for i, label in enumerate(labels):
        ax.annotate(label, (params[i] / 1e9, robustness[i]),
                    textcoords="offset points", xytext=(8, 5), fontsize=8)

    if len(params) >= 3:
        a, b, r2 = fit_log_linear(params, robustness)
        if a is not None:
            x_fit = np.linspace(params.min() * 0.8, params.max() * 1.2, 100)
            y_fit = a * np.log(x_fit) + b
            ax.plot(x_fit / 1e9, y_fit, "--", color="gray", alpha=0.7,
                    label=f"log-linear (R²={r2:.2f})")
            ax.legend(fontsize=8)

    ax.set_xscale("log")
    ax.set_xlabel("Parameters (billions)")
    ax.set_ylabel("Robustness (1 - ASR)")
    ax.set_title("Adversarial Robustness vs. Model Size")
    ax.set_ylim(-0.05, 1.05)
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    fig.savefig(output_path, bbox_inches="tight")
    print(f"Saved: {output_path}")
    plt.close()

    # Print summary table
    print("\nScaling summary:")
    print(f"{'Model':<12} {'Params':>8} {'ASR':>8} {'Robust':>8} {'AvgFit':>8}")
    for r in results:
        print(f"{r['label']:<12} {r['params']/1e9:>7.0f}B {r['asr']:>7.1%} {1-r['asr']:>7.1%} {r['avg_fitness']:>8.3f}")


def main():
    parser = argparse.ArgumentParser(description="Generate scaling curves")
    parser.add_argument("--experiments-dir", default="experiments")
    parser.add_argument("--output", default="figures/scaling_curve.pdf")
    args = parser.parse_args()

    results = load_results(args.experiments_dir)
    if not results:
        print("No Gemma results found. Run experiments first.", file=sys.stderr)
        sys.exit(1)

    plot_scaling(results, args.output)


if __name__ == "__main__":
    main()
