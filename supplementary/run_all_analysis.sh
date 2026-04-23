#!/bin/bash
# Reproduce all figures and tables from the released result JSONs.
# No GPU needed — runs on CPU in ~30 seconds.

set -eu

echo "=== Generating ASR table with CIs ==="
python3 analysis/compute_asr_with_ci.py

echo ""
echo "=== Generating transfer heatmap ==="
python3 analysis/transfer_matrix_with_ci.py

echo ""
echo "=== Generating scaling plot ==="
python3 analysis/scaling_plot.py

echo ""
echo "=== Generating category breakdown + radar ==="
python3 analysis/category_breakdown_multiseed.py

echo ""
echo "=== All figures generated in figures/ ==="
ls -la figures/*.pdf

echo ""
echo "=== All tables generated in analysis/output/ ==="
ls -la analysis/output/*
