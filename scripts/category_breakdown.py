#!/usr/bin/env python3
"""
Generate per-category vulnerability breakdown figures (Figures 2/4).

Produces:
  1. Grouped bar chart: ASR per category per Gemma model
  2. Radar/spider chart: vulnerability fingerprint per model

Usage:
    python scripts/category_breakdown.py [--experiments-dir experiments] [--output-dir figures]
"""

import argparse
import json
import math
import os
import sys

import matplotlib.pyplot as plt
import numpy as np
import matplotlib

matplotlib.rcParams.update({
    "font.family": "serif",
    "font.size": 10,
    "axes.titlesize": 12,
    "figure.dpi": 150,
})

GEMMA_MODELS = [
    ("gemma1_7b", "Gemma 1 (7B)", "#1f77b4"),
    ("gemma2_9b", "Gemma 2 (9B)", "#ff7f0e"),
    ("gemma3_12b", "Gemma 3 (12B)", "#2ca02c"),
    ("gemma4_31b", "Gemma 4 (31B)", "#d62728"),
]

CATEGORY_SHORT = {
    "chemical_biological": "Chem/Bio",
    "copyright": "Copyright",
    "cybercrime_intrusion": "Cyber",
    "harassment_bullying": "Harass",
    "harmful": "Harmful",
    "illegal": "Illegal",
    "misinformation_disinformation": "Misinfo",
    "unknown": "Other",
}


def load_category_data(experiments_dir: str) -> dict:
    """Load category_summary from each model's results."""
    data = {}
    for key, label, color in GEMMA_MODELS:
        path = os.path.join(experiments_dir, f"results_{key}.json")
        if not os.path.exists(path):
            continue
        with open(path) as f:
            results = json.load(f)
        cs = results.get("category_summary", {})
        if cs:
            data[key] = {"label": label, "color": color, "categories": cs}
    return data


def plot_grouped_bars(data: dict, output_path: str):
    """Grouped bar chart: ASR per category per model."""
    all_cats = set()
    for info in data.values():
        all_cats.update(info["categories"].keys())
    categories = sorted(all_cats)
    cat_labels = [CATEGORY_SHORT.get(c, c) for c in categories]

    n_cats = len(categories)
    n_models = len(data)
    if n_cats == 0 or n_models == 0:
        print("No category data to plot")
        return

    x = np.arange(n_cats)
    width = 0.8 / n_models

    fig, ax = plt.subplots(figsize=(10, 5))

    for idx, (key, info) in enumerate(data.items()):
        rates = [info["categories"].get(c, {}).get("success_rate", 0.0) for c in categories]
        offset = (idx - n_models / 2 + 0.5) * width
        bars = ax.bar(x + offset, rates, width, label=info["label"],
                      color=info["color"], edgecolor="black", linewidth=0.3)

    ax.set_xlabel("HarmBench Category")
    ax.set_ylabel("Attack Success Rate")
    ax.set_title("Attack Success Rate by Category Across Gemma Generations")
    ax.set_xticks(x)
    ax.set_xticklabels(cat_labels, rotation=30, ha="right")
    ax.set_ylim(0, 1.05)
    ax.legend(loc="upper right", fontsize=8)
    ax.grid(axis="y", alpha=0.3)

    plt.tight_layout()
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    fig.savefig(output_path, bbox_inches="tight")
    print(f"Saved: {output_path}")
    plt.close()


def plot_radar(data: dict, output_path: str):
    """Radar chart: vulnerability fingerprint per model."""
    all_cats = set()
    for info in data.values():
        all_cats.update(info["categories"].keys())
    categories = sorted(all_cats)
    cat_labels = [CATEGORY_SHORT.get(c, c) for c in categories]

    n_cats = len(categories)
    if n_cats < 3:
        print("Need at least 3 categories for radar chart")
        return

    angles = np.linspace(0, 2 * math.pi, n_cats, endpoint=False).tolist()
    angles += angles[:1]  # close the polygon

    fig, ax = plt.subplots(figsize=(7, 7), subplot_kw=dict(polar=True))

    for key, info in data.items():
        values = [info["categories"].get(c, {}).get("success_rate", 0.0) for c in categories]
        values += values[:1]  # close
        ax.plot(angles, values, "o-", linewidth=1.5, label=info["label"],
                color=info["color"], markersize=4)
        ax.fill(angles, values, alpha=0.1, color=info["color"])

    ax.set_xticks(angles[:-1])
    ax.set_xticklabels(cat_labels, fontsize=9)
    ax.set_ylim(0, 1.0)
    ax.set_yticks([0.25, 0.5, 0.75, 1.0])
    ax.set_yticklabels(["25%", "50%", "75%", "100%"], fontsize=7)
    ax.set_title("Vulnerability Fingerprint by Category", pad=20)
    ax.legend(loc="upper right", bbox_to_anchor=(1.3, 1.1), fontsize=8)

    plt.tight_layout()
    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    fig.savefig(output_path, bbox_inches="tight")
    print(f"Saved: {output_path}")
    plt.close()


def print_table(data: dict):
    """Print a LaTeX-ready table of per-category results."""
    all_cats = set()
    for info in data.values():
        all_cats.update(info["categories"].keys())
    categories = sorted(all_cats)

    print("\nPer-category ASR table:")
    header = f"{'Category':<20}" + "".join(f"{info['label']:>16}" for info in data.values())
    print(header)
    print("-" * len(header))

    for cat in categories:
        row = f"{CATEGORY_SHORT.get(cat, cat):<20}"
        for info in data.values():
            cs = info["categories"].get(cat, {})
            rate = cs.get("success_rate", 0.0)
            row += f"{rate:>15.1%} "
        print(row)

    # Overall
    row = f"{'Overall':<20}"
    for info in data.values():
        total = sum(v["total_attacks"] for v in info["categories"].values())
        success = sum(v["successful_attacks"] for v in info["categories"].values())
        rate = success / total if total > 0 else 0.0
        row += f"{rate:>15.1%} "
    print(row)


def main():
    parser = argparse.ArgumentParser(description="Generate category breakdown figures")
    parser.add_argument("--experiments-dir", default="experiments")
    parser.add_argument("--output-dir", default="figures")
    args = parser.parse_args()

    data = load_category_data(args.experiments_dir)
    if not data:
        print("No category data found. Run experiments first.", file=sys.stderr)
        sys.exit(1)

    plot_grouped_bars(data, os.path.join(args.output_dir, "category_breakdown.pdf"))
    plot_radar(data, os.path.join(args.output_dir, "vulnerability_radar.pdf"))
    print_table(data)


if __name__ == "__main__":
    main()
