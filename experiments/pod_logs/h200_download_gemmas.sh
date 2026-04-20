#!/bin/bash
set -u
export HF_HUB_ENABLE_HF_TRANSFER=1
export HF_HOME=/workspace/hf-cache
export HF_TOKEN="$(cat /root/.cache/huggingface/token)"
LOG=/workspace/download.log
echo "===START $(date -Iseconds)===" > "$LOG"
for m in "google/gemma-7b-it" "google/gemma-2-9b-it" "google/gemma-3-12b-it" "google/gemma-4-31B-it"; do
  echo "=== downloading $m at $(date -Iseconds) ===" | tee -a "$LOG"
  hf download "$m" --include "*.safetensors" "*.json" "tokenizer.model" 2>&1 | tail -3 >> "$LOG"
  rc=$?
  folder="/workspace/hf-cache/hub/models--${m//\//--}"
  size=$(du -sh "$folder" 2>/dev/null | cut -f1)
  echo "=== done $m exit=$rc size=$size ===" | tee -a "$LOG"
done
echo "===ALL_DONE $(date -Iseconds)===" | tee -a "$LOG"
du -sh /workspace/hf-cache >> "$LOG"
