#!/bin/bash
#
# Fix 4: Raw HarmBench baseline.
# Sends all 400 seed prompts directly to each Gemma model (no evolution)
# and scores with the same Llama judge. Produces per-model ASR baseline.
#
# Usage: ./scripts/run_harmbench_baseline.sh

set -euo pipefail

SSH_KEY="$HOME/.ssh/neurips-a100"
TARGET_HOST="root@38.80.152.249"
TARGET_PORT="30959"
TARGET_SSH="-i $SSH_KEY -o StrictHostKeyChecking=no -o ConnectTimeout=30 -p $TARGET_PORT"
JUDGE_HOST="root@213.173.98.91"
JUDGE_PORT="15673"
JUDGE_SSH="-i $SSH_KEY -o StrictHostKeyChecking=no -o ConnectTimeout=30 -p $JUDGE_PORT"

LOCAL_TARGET_PORT=8000
LOCAL_JUDGE_PORT=8001

JUDGE="openai-compat:meta-llama/Llama-3.1-8B-Instruct@http://localhost:${LOCAL_JUDGE_PORT}"
SEEDS_FILE="experiments/seeds/harmbench.jsonl"
OUTDIR="results/baseline"

MODELS=(
  "google/gemma-7b-it"
  "google/gemma-2-9b-it"
  "google/gemma-3-12b-it"
  "google/gemma-4-31B-it"
)
MODEL_NAMES=(
  "gemma1_7b"
  "gemma2_9b"
  "gemma3_12b"
  "gemma4_31b"
)

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }
ssh_target() { ssh $TARGET_SSH "$TARGET_HOST" "$@"; }

mkdir -p "$OUTDIR"

# Run a minimal 1-generation, batch=400 MAP-Elites (effectively just evaluates seeds)
for i in "${!MODELS[@]}"; do
  model="${MODELS[$i]}"
  name="${MODEL_NAMES[$i]}"
  output="$OUTDIR/baseline_${name}.json"

  if [ -f "$output" ]; then
    log "SKIP: $output exists"
    continue
  fi

  log "=== Baseline: $model ==="

  # Kill existing vLLM + swap
  ssh_target 'for pid in $(nvidia-smi --query-compute-apps=pid --format=csv,noheader 2>/dev/null); do kill -9 "$pid" 2>/dev/null; done; sleep 2' || true
  ssh_target "cat > /workspace/serve.sh" <<'SCRIPT'
#!/bin/bash
set -eu
MODEL="${1:?}"
PORT="${2:-8000}"
export HF_HOME=/workspace/hf-cache
python3 -m vllm.entrypoints.openai.api_server --model "$MODEL" --dtype bfloat16 --host 0.0.0.0 --port "$PORT" --max-model-len 4096 --gpu-memory-utilization 0.90 --trust-remote-code 2>&1 | tee /workspace/vllm_server.log
SCRIPT
  ssh_target "chmod +x /workspace/serve.sh && nohup /workspace/serve.sh '$model' 8000 > /dev/null 2>&1 & disown; echo launched"

  # Wait for model
  log "Waiting for $model..."
  for j in $(seq 1 60); do
    if (curl -sS "http://localhost:${LOCAL_TARGET_PORT}/v1/models" 2>/dev/null | grep -q "${model}") 2>/dev/null; then
      log "Ready: $model"
      break
    fi
    sleep 10
  done

  # Run 1-generation evolution (effectively just evaluates the 400 seeds)
  cargo run --release --quiet -- llm jailbreak \
    --target "openai-compat:${model}@http://localhost:${LOCAL_TARGET_PORT}" \
    --seeds "$SEEDS_FILE" \
    --judge "$JUDGE" \
    --seed 42 \
    --map-elites --me-bins 6 --me-batch-size 400 --generations 1 \
    --store-responses --concurrency 5 \
    --output "$output"

  log "DONE: $output"
done

log "=== All baselines done ==="
