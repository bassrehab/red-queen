#!/usr/bin/env python3
"""
Phase 2B: Transfer matrix with bootstrap CIs and significance tests.

Produces updated transfer heatmap with mean ± CI in each cell.
"""

import json
import os
import csv
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
    ("gemma1_7b", "Gemma 1\n(7B)"),
    ("gemma2_9b", "Gemma 2\n(9B)"),
    ("gemma3_12b", "Gemma 3\n(12B)"),
    ("gemma4_31b", "Gemma 4\n(31B)"),
]
SEEDS = [42, 1337, 2718]
RESULTS_DIR = "results"
N_BOOTSTRAP = 10000
RNG = np.random.RandomState(42)


def load_self_rates():
    """Load self-ASR per (model, seed)."""
    rates = {}
    for model_key, _ in MODELS:
        rates[model_key] = []
        for seed in SEEDS:
            path = os.path.join(RESULTS_DIR, model_key, f"seed_{seed}", "results.json")
            if not os.path.exists(path):
                continue
            with open(path) as f:
                d = json.load(f)
            cs = d.get("category_summary", {})
            total = sum(v["total_attacks"] for v in cs.values())
            success = sum(v["successful_attacks"] for v in cs.values())
            rates[model_key].append(success / total if total > 0 else 0.0)
    return rates


def load_transfer_rates():
    """Load transfer ASR per (source, target, seed)."""
    rates = {}
    for src_key, _ in MODELS:
        for dst_key, _ in MODELS:
            if src_key == dst_key:
                continue
            rates[(src_key, dst_key)] = []
            for seed in SEEDS:
                path = os.path.join(
                    RESULTS_DIR, "transfer",
                    f"{src_key}_seed{seed}_on_{dst_key}.json"
                )
                if not os.path.exists(path):
                    continue
                with open(path) as f:
                    d = json.load(f)
                rates[(src_key, dst_key)].append(d["overall_success_rate"])
    return rates


def bootstrap_ci(values, n=N_BOOTSTRAP):
    values = np.array(values)
    if len(values) < 2:
        return values.mean(), 0.0, values.mean(), values.mean()
    boots = np.array([RNG.choice(values, size=len(values), replace=True).mean() for _ in range(n)])
    return values.mean(), values.std(ddof=1), np.percentile(boots, 2.5), np.percentile(boots, 97.5)


def main():
    self_rates = load_self_rates()
    transfer_rates = load_transfer_rates()
    n = len(MODELS)

    # Build matrices
    mean_matrix = np.full((n, n), np.nan)
    ci_half = np.full((n, n), np.nan)
    std_matrix = np.full((n, n), np.nan)

    csv_rows = []

    for i, (src_key, _) in enumerate(MODELS):
        for j, (dst_key, _) in enumerate(MODELS):
            if i == j:
                vals = self_rates[src_key]
            else:
                vals = transfer_rates.get((src_key, dst_key), [])

            if vals:
                mean, std, ci_low, ci_high = bootstrap_ci(vals)
                mean_matrix[i, j] = mean
                std_matrix[i, j] = std
                ci_half[i, j] = (ci_high - ci_low) / 2
                csv_rows.append({
                    "source": src_key,
                    "target": dst_key,
                    "mean": mean,
                    "std": std,
                    "ci_low": ci_low,
                    "ci_high": ci_high,
                    "n_seeds": len(vals),
                })

    # Save CSV
    csv_path = "analysis/output/transfer_matrix.csv"
    with open(csv_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=[
            "source", "target", "mean", "std", "ci_low", "ci_high", "n_seeds"
        ])
        writer.writeheader()
        writer.writerows(csv_rows)
    print(f"CSV saved: {csv_path}")

    # Print matrix
    labels_short = [k for k, _ in MODELS]
    print("\nTransfer matrix (mean ± std):")
    print(f"{'':>12}", end="")
    for l in labels_short:
        print(f"{l:>16}", end="")
    print()
    for i, l in enumerate(labels_short):
        print(f"{l:>12}", end="")
        for j in range(n):
            m = mean_matrix[i, j]
            s = std_matrix[i, j]
            if np.isnan(m):
                print(f"{'---':>16}", end="")
            else:
                diag = "*" if i == j else " "
                print(f"{m*100:>6.1f}±{s*100:4.1f}%{diag} ", end="")
        print()

    # Key asymmetry test
    print("\nKey asymmetry: Gemma 1 → Gemma 3 vs Gemma 3 → Gemma 1")
    fwd = transfer_rates.get(("gemma1_7b", "gemma3_12b"), [])
    bwd = transfer_rates.get(("gemma3_12b", "gemma1_7b"), [])
    if fwd and bwd:
        diff = np.mean(fwd) - np.mean(bwd)
        # Permutation test
        combined = np.array(fwd + bwd)
        nf = len(fwd)
        count = 0
        for _ in range(N_BOOTSTRAP):
            perm = RNG.permutation(combined)
            d = perm[:nf].mean() - perm[nf:].mean()
            if abs(d) >= abs(diff):
                count += 1
        p = count / N_BOOTSTRAP
        print(f"  Forward: {np.mean(fwd):.1%}, Backward: {np.mean(bwd):.1%}, diff={diff:+.1%}, p={p:.4f}")

    # Plot heatmap with CIs
    labels = [label for _, label in MODELS]

    fig, ax = plt.subplots(figsize=(6.5, 5.5))
    im = ax.imshow(mean_matrix, cmap="RdYlGn_r", vmin=0, vmax=1, aspect="equal")

    ax.set_xticks(range(n))
    ax.set_xticklabels(labels, ha="center")
    ax.set_yticks(range(n))
    ax.set_yticklabels(labels)
    ax.set_xlabel("Tested on (target)")
    ax.set_ylabel("Evolved on (source)")
    ax.set_title("Cross-Generational Attack Transfer Rate")

    for i in range(n):
        for j in range(n):
            m = mean_matrix[i, j]
            c = ci_half[i, j]
            if not np.isnan(m):
                color = "white" if m > 0.5 else "black"
                fontweight = "bold" if i == j else "normal"
                text = f"{m:.0%}\n±{s:.0%}"
                ax.text(j, i, text, ha="center", va="center",
                        color=color, fontweight=fontweight, fontsize=8)

    cbar = fig.colorbar(im, ax=ax, label="Attack Success Rate", shrink=0.8)
    cbar.set_ticks([0, 0.25, 0.5, 0.75, 1.0])
    cbar.set_ticklabels(["0%", "25%", "50%", "75%", "100%"])

    fig.text(0.5, 0.01, "Cells show mean ASR ± std across 3 seeds",
             ha="center", fontsize=7, style="italic")

    plt.tight_layout()
    out = "figures/transfer_heatmap.pdf"
    os.makedirs(os.path.dirname(out), exist_ok=True)
    fig.savefig(out, bbox_inches="tight")
    print(f"\nFigure saved: {out}")
    plt.close()


if __name__ == "__main__":
    main()
