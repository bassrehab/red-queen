#!/usr/bin/env python3
"""
Generate the cross-generational transfer heatmap (Figure 1).

Reads transfer_*.json files from the experiments directory and produces
a 4x4 heatmap showing attack success rate when replaying archives
evolved on model_i against model_j.

Usage:
    python scripts/transfer_matrix.py [--experiments-dir experiments] [--output figures/transfer_heatmap.pdf]
"""

import argparse
import json
import os
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
import matplotlib

matplotlib.rcParams.update({
    "font.family": "serif",
    "font.size": 10,
    "axes.titlesize": 12,
    "figure.dpi": 150,
})

MODEL_ORDER = [
    ("gemma1_7b", "Gemma 1\n(7B)"),
    ("gemma2_9b", "Gemma 2\n(9B)"),
    ("gemma3_12b", "Gemma 3\n(12B)"),
    ("gemma4_31b", "Gemma 4\n(31B)"),
]


def load_transfer_results(experiments_dir: str) -> dict:
    """Load all transfer_*_to_*.json files into a matrix dict."""
    results = {}
    for src_key, _ in MODEL_ORDER:
        for dst_key, _ in MODEL_ORDER:
            if src_key == dst_key:
                continue
            path = os.path.join(experiments_dir, f"transfer_{src_key}_to_{dst_key}.json")
            if os.path.exists(path):
                with open(path) as f:
                    data = json.load(f)
                results[(src_key, dst_key)] = data
    return results


def load_self_rates(experiments_dir: str) -> dict:
    """Load self-evaluation success rates from results_*.json category_summary."""
    rates = {}
    for key, _ in MODEL_ORDER:
        path = os.path.join(experiments_dir, f"results_{key}.json")
        if os.path.exists(path):
            with open(path) as f:
                data = json.load(f)
            cs = data.get("category_summary", {})
            if cs:
                total = sum(v["total_attacks"] for v in cs.values())
                success = sum(v["successful_attacks"] for v in cs.values())
                rates[key] = success / total if total > 0 else 0.0
            else:
                best = data.get("best_attack", {}).get("fitness", 0.0)
                rates[key] = best
    return rates


def build_matrix(transfer_results: dict, self_rates: dict) -> np.ndarray:
    """Build the NxN transfer matrix."""
    n = len(MODEL_ORDER)
    matrix = np.full((n, n), np.nan)

    for i, (src_key, _) in enumerate(MODEL_ORDER):
        for j, (dst_key, _) in enumerate(MODEL_ORDER):
            if i == j:
                matrix[i, j] = self_rates.get(src_key, np.nan)
            else:
                data = transfer_results.get((src_key, dst_key))
                if data:
                    matrix[i, j] = data["overall_success_rate"]

    return matrix


def plot_heatmap(matrix: np.ndarray, output_path: str):
    """Plot the transfer heatmap."""
    n = len(MODEL_ORDER)
    labels = [label for _, label in MODEL_ORDER]

    fig, ax = plt.subplots(figsize=(6, 5))

    im = ax.imshow(matrix, cmap="RdYlGn_r", vmin=0, vmax=1, aspect="equal")

    ax.set_xticks(range(n))
    ax.set_xticklabels(labels, ha="center")
    ax.set_yticks(range(n))
    ax.set_yticklabels(labels)

    ax.set_xlabel("Tested on (target)")
    ax.set_ylabel("Evolved on (source)")
    ax.set_title("Cross-Generational Attack Transfer Rate")

    for i in range(n):
        for j in range(n):
            val = matrix[i, j]
            if not np.isnan(val):
                color = "white" if val > 0.6 else "black"
                text = f"{val:.0%}" if i == j else f"{val:.0%}"
                fontweight = "bold" if i == j else "normal"
                ax.text(j, i, text, ha="center", va="center",
                        color=color, fontweight=fontweight, fontsize=9)

    cbar = fig.colorbar(im, ax=ax, label="Attack Success Rate", shrink=0.8)
    cbar.set_ticks([0, 0.25, 0.5, 0.75, 1.0])
    cbar.set_ticklabels(["0%", "25%", "50%", "75%", "100%"])

    plt.tight_layout()
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    fig.savefig(output_path, bbox_inches="tight")
    print(f"Saved: {output_path}")
    plt.close()


def plot_category_transfer(transfer_results: dict, output_path: str):
    """Plot per-category transfer rates as a grouped heatmap."""
    categories = set()
    for data in transfer_results.values():
        categories.update(data.get("category_transfer", {}).keys())
    categories = sorted(categories)

    if not categories:
        print("No category data available for category transfer plot")
        return

    n_models = len(MODEL_ORDER)
    n_cats = len(categories)

    # Build forward-transfer matrix (i < j): averaged across categories
    fig, ax = plt.subplots(figsize=(10, 5))

    # For each source→dest pair, show category-level transfer
    pairs = []
    cat_rates = []
    for i, (src_key, src_label) in enumerate(MODEL_ORDER):
        for j, (dst_key, dst_label) in enumerate(MODEL_ORDER):
            if i >= j:
                continue
            data = transfer_results.get((src_key, dst_key))
            if not data:
                continue
            ct = data.get("category_transfer", {})
            pairs.append(f"{src_label.replace(chr(10),' ')} → {dst_label.replace(chr(10),' ')}")
            rates = [ct.get(c, {}).get("rate", 0.0) for c in categories]
            cat_rates.append(rates)

    if not cat_rates:
        print("No forward-transfer pairs found")
        return

    cat_rates = np.array(cat_rates)

    im = ax.imshow(cat_rates, cmap="RdYlGn_r", vmin=0, vmax=1, aspect="auto")

    ax.set_xticks(range(n_cats))
    ax.set_xticklabels([c.replace("_", "\n") for c in categories], fontsize=7, ha="center")
    ax.set_yticks(range(len(pairs)))
    ax.set_yticklabels(pairs, fontsize=8)

    for i in range(len(pairs)):
        for j in range(n_cats):
            ax.text(j, i, f"{cat_rates[i, j]:.0%}", ha="center", va="center",
                    fontsize=7, color="white" if cat_rates[i, j] > 0.6 else "black")

    ax.set_title("Category-Conditional Forward Transfer Rate")
    fig.colorbar(im, ax=ax, label="Transfer Rate", shrink=0.8)

    plt.tight_layout()
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    fig.savefig(output_path, bbox_inches="tight")
    print(f"Saved: {output_path}")
    plt.close()


def main():
    parser = argparse.ArgumentParser(description="Generate transfer heatmaps")
    parser.add_argument("--experiments-dir", default="experiments")
    parser.add_argument("--output", default="figures/transfer_heatmap.pdf")
    parser.add_argument("--category-output", default="figures/category_transfer.pdf")
    args = parser.parse_args()

    transfer_results = load_transfer_results(args.experiments_dir)
    self_rates = load_self_rates(args.experiments_dir)

    if not transfer_results and not self_rates:
        print("No results found. Run experiments first.", file=sys.stderr)
        sys.exit(1)

    matrix = build_matrix(transfer_results, self_rates)
    print("Transfer matrix:")
    keys = [k for k, _ in MODEL_ORDER]
    print(f"{'':>12}", end="")
    for k in keys:
        print(f"{k:>12}", end="")
    print()
    for i, k in enumerate(keys):
        print(f"{k:>12}", end="")
        for j in range(len(keys)):
            v = matrix[i, j]
            print(f"{'N/A':>12}" if np.isnan(v) else f"{v:>11.1%} ", end="")
        print()
    print()

    plot_heatmap(matrix, args.output)

    if transfer_results:
        plot_category_transfer(transfer_results, args.category_output)


if __name__ == "__main__":
    main()
