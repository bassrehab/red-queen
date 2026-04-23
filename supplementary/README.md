# Supplementary Material — Reproducibility Guide

## Overview

This package contains all code, analysis scripts, and seed-level result
artifacts needed to reproduce the experiments and figures in
"Cross-Generational Transfer of Adversarial Attacks Reveals Non-Monotonic
Safety Alignment in LLMs."

## Repository Structure

```
.
├── crates/                    # Rust MAP-Elites evolution framework
│   ├── red-queen-core/        # Core evolutionary algorithms
│   ├── red-queen-llm/         # LLM target adapters, fitness, judge
│   └── red-queen-cli/         # CLI: jailbreak, transfer commands
├── analysis/                  # Python scripts producing tables and figures
│   ├── compute_asr_with_ci.py          # Table 2: ASR with bootstrap CIs
│   ├── transfer_matrix_with_ci.py      # Figure 1: transfer heatmap
│   ├── scaling_plot.py                 # Figure 3: robustness vs size
│   └── category_breakdown_multiseed.py # Figures 2, 4: bars + radar
├── results/                   # Experiment artifacts (seed-level)
│   ├── gemma1_7b/seed_{42,1337,2718}/  # Evolution results + archives
│   ├── gemma2_9b/seed_{42,1337,2718}/
│   ├── gemma3_12b/seed_{42,1337,2718}/
│   ├── gemma4_31b/seed_{42,1337,2718}/
│   └── transfer/              # 36 cross-model transfer replays
├── experiments/seeds/         # HarmBench seed behaviors (400 JSONL)
├── scripts/                   # Orchestration scripts
│   └── run_gemma_sweep.sh     # Full sweep automation
├── figures/                   # Generated PDF figures
└── paper/neurips/             # LaTeX source
```

## Software Versions

| Component | Version |
|-----------|---------|
| Rust | 1.83+ |
| vLLM | 0.19.1 |
| transformers | 5.5.4 (for Gemma 4) |
| Python | 3.11 |
| matplotlib | 3.x |
| numpy | 1.x |
| scipy | 1.x |
| CUDA Driver | 580.126.16 |

## Reproducing Figures and Tables from Released Results

No GPU needed — all results are included.

```bash
# Install Python dependencies
pip install matplotlib numpy scipy

# Generate all figures and tables
python analysis/compute_asr_with_ci.py           # → analysis/output/asr_summary.{csv,tex}
python analysis/transfer_matrix_with_ci.py        # → figures/transfer_heatmap.pdf
python analysis/scaling_plot.py                   # → figures/scaling_curve.pdf
python analysis/category_breakdown_multiseed.py   # → figures/category_breakdown.pdf, vulnerability_radar.pdf

# Compile paper
cd paper/neurips && pdflatex main.tex && bibtex main && pdflatex main.tex && pdflatex main.tex
```

## Reproducing Experiments from Scratch

Requires two GPUs: one ≥80 GB for Gemma targets, one ≥24 GB for the Llama judge.

### 1. Build the framework

```bash
cargo build --release
```

### 2. Set up model serving

On the target GPU:
```bash
pip install vllm==0.19.1
python -m vllm.entrypoints.openai.api_server \
  --model google/gemma-7b-it --dtype bfloat16 --host 0.0.0.0 --port 8000 \
  --max-model-len 4096 --gpu-memory-utilization 0.90 --trust-remote-code
```

On the judge GPU:
```bash
python -m vllm.entrypoints.openai.api_server \
  --model meta-llama/Llama-3.1-8B-Instruct --dtype bfloat16 --host 0.0.0.0 --port 8000 \
  --max-model-len 4096 --gpu-memory-utilization 0.90 --trust-remote-code
```

### 3. Run a single evolution

```bash
cargo run --release -- llm jailbreak \
  --target "openai-compat:google/gemma-7b-it@http://<target-host>:8000" \
  --judge "openai-compat:meta-llama/Llama-3.1-8B-Instruct@http://<judge-host>:8000" \
  --seeds experiments/seeds/harmbench.jsonl \
  --seed 42 \
  --map-elites --me-bins 6 --me-batch-size 50 --generations 200 \
  --store-responses --concurrency 5 \
  --output results/gemma1_7b/seed_42/results.json
```

### 4. Run a transfer replay

```bash
cargo run --release -- llm transfer \
  --archive results/gemma1_7b/seed_42/results_archive.json \
  --target "openai-compat:google/gemma-2-9b-it@http://<target-host>:8000" \
  --judge "openai-compat:meta-llama/Llama-3.1-8B-Instruct@http://<judge-host>:8000" \
  --concurrency 5 \
  --output results/transfer/gemma1_7b_seed42_on_gemma2_9b.json
```

### 5. Full automated sweep

```bash
# Edit scripts/run_gemma_sweep.sh with your SSH/host details, then:
./scripts/run_gemma_sweep.sh
```

This runs all 12 evolution + 36 transfer replay jobs sequentially,
swapping models on the target GPU automatically.

## What Is NOT Included

- **Raw attack prompts and model responses** are not included in the
  released results to prevent misuse. The result JSONs contain aggregate
  statistics (ASR per category, transfer rates) but not the specific
  prompts that achieved jailbreaks.
- **Model weights** are available from HuggingFace under Google's Gemma
  license. The HarmBench seed behaviors are from the public HarmBench
  dataset.

## Estimated Compute

- 12 evolution runs × ~4 hours each = ~48 GPU-hours (target)
- 36 transfer replays × ~8 minutes each = ~5 GPU-hours (target)
- Judge runs concurrently on a separate, smaller GPU
- **Total: ~55 GPU-hours**
