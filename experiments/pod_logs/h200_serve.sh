#!/bin/bash
set -eu

MODEL="${1:?Usage: ./serve.sh MODEL_ID [PORT]}"
PORT="${2:-8000}"

export HF_HOME=/workspace/hf-cache
export HF_TOKEN="$(cat /root/.cache/huggingface/token)"

echo "=== Starting vLLM for $MODEL on port $PORT ==="
echo "  HF_HOME=$HF_HOME"
echo "  $(date -Iseconds)"

python3 -m vllm.entrypoints.openai.api_server \
  --model "$MODEL" \
  --dtype bfloat16 \
  --host 0.0.0.0 \
  --port "$PORT" \
  --max-model-len 4096 \
  --gpu-memory-utilization 0.90 \
  --trust-remote-code \
  2>&1 | tee /workspace/vllm_server.log
