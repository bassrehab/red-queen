#!/usr/bin/env python3
"""
Phase 2A supplement: Category breakdown with multi-seed error bars.
"""

import json
import os
import math
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
    ("gemma1_7b", "Gemma 1 (7B)", "#1f77b4"),
    ("gemma2_9b", "Gemma 2 (9B)", "#ff7f0e"),
    ("gemma3_12b", "Gemma 3 (12B)", "#2ca02c"),
    ("gemma4_31b", "Gemma 4 (31B)", "#d62728"),
]
SEEDS = [42, 1337, 2718]
RESULTS_DIR = "results"
CATEGORY_SHORT = {
    "chemical_biological": "Chem/Bio",
    "copyright": "Copyright",
    "cybercrime_intrusion": "Cyber",
    "harassment_bullying": "Harass",
    "harmful": "Harmful",
    "illegal": "Illegal",
    "misinformation_disinformation": "Misinfo",
}


def load_data():
    data = {}
    for key, label, color in MODELS:
        cat_rates = {}
        for seed in SEEDS:
            path = os.path.join(RESULTS_DIR, key, f"seed_{seed}", "results.json")
            if not os.path.exists(path):
                continue
            with open(path) as f:
                d = json.load(f)
            for cat, v in d.get("category_summary", {}).items():
                if cat not in cat_rates:
                    cat_rates[cat] = []
                cat_rates[cat].append(v["success_rate"])
        data[key] = {"label": label, "color": color, "categories": cat_rates}
    return data


def main():
    data = load_data()
    all_cats = set()
    for info in data.values():
        all_cats.update(info["categories"].keys())
    categories = sorted(all_cats)
    cat_labels = [CATEGORY_SHORT.get(c, c) for c in categories]
    n_cats = len(categories)
    n_models = len(data)

    # Grouped bar chart with error bars
    x = np.arange(n_cats)
    width = 0.8 / n_models

    fig, ax = plt.subplots(figsize=(10, 5))
    for idx, (key, info) in enumerate(data.items()):
        means = [np.mean(info["categories"].get(c, [0])) for c in categories]
        stds = [np.std(info["categories"].get(c, [0]), ddof=1) if len(info["categories"].get(c, [])) > 1 else 0 for c in categories]
        offset = (idx - n_models / 2 + 0.5) * width
        ax.bar(x + offset, means, width, yerr=stds, label=info["label"],
               color=info["color"], edgecolor="black", linewidth=0.3,
               capsize=2, error_kw={"linewidth": 0.8})

    ax.set_xlabel("HarmBench Category")
    ax.set_ylabel("Attack Success Rate")
    ax.set_title("Attack Success Rate by Category (mean ± std, 3 seeds)")
    ax.set_xticks(x)
    ax.set_xticklabels(cat_labels, rotation=30, ha="right")
    ax.set_ylim(0, 1.15)
    ax.legend(loc="upper right", fontsize=8)
    ax.grid(axis="y", alpha=0.3)

    plt.tight_layout()
    os.makedirs("figures", exist_ok=True)
    fig.savefig("figures/category_breakdown.pdf", bbox_inches="tight")
    print("Saved: figures/category_breakdown.pdf")
    plt.close()

    # Radar chart
    if n_cats >= 3:
        angles = np.linspace(0, 2 * math.pi, n_cats, endpoint=False).tolist()
        angles += angles[:1]

        fig, ax = plt.subplots(figsize=(7, 7), subplot_kw=dict(polar=True))
        for key, info in data.items():
            means = [np.mean(info["categories"].get(c, [0])) for c in categories]
            means += means[:1]
            ax.plot(angles, means, "o-", linewidth=1.5, label=info["label"],
                    color=info["color"], markersize=4)
            ax.fill(angles, means, alpha=0.1, color=info["color"])

        ax.set_xticks(angles[:-1])
        ax.set_xticklabels(cat_labels, fontsize=9)
        ax.set_ylim(0, 1.0)
        ax.set_yticks([0.25, 0.5, 0.75, 1.0])
        ax.set_yticklabels(["25%", "50%", "75%", "100%"], fontsize=7)
        ax.set_title("Vulnerability Fingerprint (mean, 3 seeds)", pad=20)
        ax.legend(loc="upper right", bbox_to_anchor=(1.3, 1.1), fontsize=8)

        plt.tight_layout()
        fig.savefig("figures/vulnerability_radar.pdf", bbox_inches="tight")
        print("Saved: figures/vulnerability_radar.pdf")
        plt.close()


if __name__ == "__main__":
    main()
