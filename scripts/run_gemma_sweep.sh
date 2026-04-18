#!/bin/bash
#
# Orchestrates the full Gemma generational sweep + transfer replay.
# Runs unattended: swaps vLLM models on the RunPod pod, runs evolution
# locally via SSH tunnel, then replays archives across all model pairs.
#
# Prerequisites:
#   - SSH tunnel or direct access to RunPod pod
#   - All Gemma weights pre-downloaded on pod at /workspace/hf-cache
#   - OPENAI_API_KEY set locally (for LLM judge)
#   - red-queen built: cargo build --release
#
# Usage:
#   ./scripts/run_gemma_sweep.sh [--skip-evolution] [--skip-transfer]

set -euo pipefail

# --- Configuration -----------------------------------------------------------

SSH_KEY="$HOME/.ssh/neurips-a100"
SSH_HOST="root@154.54.101.58"
SSH_PORT="18478"
SSH_OPTS="-i $SSH_KEY -o StrictHostKeyChecking=no -o ConnectTimeout=30 -p $SSH_PORT"

VLLM_PORT=8000
LOCAL_TUNNEL_PORT=8000

JUDGE="openai:gpt-5.4-mini"
SEEDS="experiments/seeds/harmbench.jsonl"
OUTDIR="experiments"

# MAP-Elites config
ME_BINS=6
ME_BATCH=50
GENERATIONS=200
CONCURRENCY=5

# Models to sweep (order matters — smallest first for fastest validation)
MODELS=(
  "google/gemma-7b-it"
  "google/gemma-2-9b-it"
  "google/gemma-3-12b-it"
  "google/gemma-4-31B-it"
)

# Short names for output files
MODEL_NAMES=(
  "gemma1_7b"
  "gemma2_9b"
  "gemma3_12b"
  "gemma4_31b"
)

# --- Flags --------------------------------------------------------------------

SKIP_EVOLUTION=false
SKIP_TRANSFER=false

for arg in "$@"; do
  case $arg in
    --skip-evolution) SKIP_EVOLUTION=true ;;
    --skip-transfer)  SKIP_TRANSFER=true ;;
    *) echo "Unknown flag: $arg"; exit 1 ;;
  esac
done

# --- Helpers ------------------------------------------------------------------

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }

ssh_pod() { ssh $SSH_OPTS "$SSH_HOST" "$@"; }

kill_tunnel() {
  # Kill any existing SSH tunnels on the local port
  lsof -ti tcp:$LOCAL_TUNNEL_PORT 2>/dev/null | xargs kill -9 2>/dev/null || true
}

start_tunnel() {
  kill_tunnel
  ssh $SSH_OPTS -o ServerAliveInterval=60 -o ServerAliveCountMax=5 \
    -o ExitOnForwardFailure=yes \
    -L ${LOCAL_TUNNEL_PORT}:localhost:${VLLM_PORT} -fN "$SSH_HOST"
  log "SSH tunnel active on localhost:$LOCAL_TUNNEL_PORT (keepalive=60s)"
}

wait_for_vllm() {
  local model="$1"
  local max_wait=300  # 5 minutes
  local elapsed=0

  log "Waiting for vLLM to serve $model..."
  while [ $elapsed -lt $max_wait ]; do
    if curl -sS "http://localhost:${LOCAL_TUNNEL_PORT}/v1/models" 2>/dev/null | grep -q "$model"; then
      log "vLLM ready: $model"
      return 0
    fi
    sleep 5
    elapsed=$((elapsed + 5))
  done

  log "ERROR: vLLM did not start within ${max_wait}s"
  return 1
}

swap_model() {
  local model="$1"
  log "Swapping vLLM to $model..."

  # Kill existing vLLM
  ssh_pod 'pkill -f "vllm.entrypoints" 2>/dev/null || true; sleep 3; pkill -9 -f "vllm.entrypoints" 2>/dev/null || true; sleep 2'

  # Start new model
  ssh_pod "nohup /workspace/serve.sh '$model' $VLLM_PORT > /dev/null 2>&1 & disown; echo 'launched'"

  # Ensure tunnel is alive (model swap may have disrupted it)
  start_tunnel

  # Wait for server
  wait_for_vllm "$model"
}

# --- Phase 1: Evolution runs -------------------------------------------------

run_evolution() {
  local model="$1"
  local name="$2"
  local output="$OUTDIR/results_${name}.json"

  if [ -f "$output" ]; then
    log "SKIP: $output already exists"
    return 0
  fi

  log "=== Evolution: $model → $output ==="

  swap_model "$model"

  cargo run --release --quiet -- llm jailbreak \
    --target "openai-compat:${model}@http://localhost:${LOCAL_TUNNEL_PORT}" \
    --seeds "$SEEDS" \
    --judge "$JUDGE" \
    --map-elites --me-bins $ME_BINS --me-batch-size $ME_BATCH --generations $GENERATIONS \
    --store-responses --concurrency $CONCURRENCY \
    --output "$output"

  if [ -f "$output" ]; then
    log "DONE: $output"
  else
    log "ERROR: $output not created"
    return 1
  fi
}

# --- Phase 2: Transfer replay ------------------------------------------------

run_transfer() {
  local src_idx=$1
  local dst_idx=$2
  local src_name="${MODEL_NAMES[$src_idx]}"
  local dst_name="${MODEL_NAMES[$dst_idx]}"
  local dst_model="${MODELS[$dst_idx]}"
  local archive="$OUTDIR/results_${src_name}_archive.json"
  local output="$OUTDIR/transfer_${src_name}_to_${dst_name}.json"

  if [ -f "$output" ]; then
    log "SKIP: $output already exists"
    return 0
  fi

  if [ ! -f "$archive" ]; then
    log "SKIP: archive $archive not found"
    return 0
  fi

  log "=== Transfer: $src_name → $dst_name ==="

  cargo run --release --quiet -- llm transfer \
    --archive "$archive" \
    --target "openai-compat:${dst_model}@http://localhost:${LOCAL_TUNNEL_PORT}" \
    --judge "$JUDGE" \
    --concurrency $CONCURRENCY \
    --output "$output"

  log "DONE: $output"
}

# --- Main ---------------------------------------------------------------------

log "Red Queen — Gemma Generational Sweep"
log "Models: ${MODELS[*]}"
log "Config: $GENERATIONS generations, batch $ME_BATCH, bins $ME_BINS"
log "Judge: $JUDGE"
log ""

mkdir -p "$OUTDIR"

# Ensure tunnel is up
start_tunnel

if [ "$SKIP_EVOLUTION" = false ]; then
  log "=== Phase 1: Evolution runs ==="
  for i in "${!MODELS[@]}"; do
    run_evolution "${MODELS[$i]}" "${MODEL_NAMES[$i]}"
  done
  log "=== Phase 1 complete ==="
else
  log "Skipping evolution (--skip-evolution)"
fi

if [ "$SKIP_TRANSFER" = false ]; then
  log "=== Phase 2: Transfer replay ==="

  # For each pair (i, j) where i != j:
  # Load model j on vLLM, then replay all archives from other models against it.
  # This minimizes model swaps: load each model once, replay all source archives.
  for j in "${!MODELS[@]}"; do
    swap_model "${MODELS[$j]}"

    for i in "${!MODELS[@]}"; do
      if [ "$i" -ne "$j" ]; then
        run_transfer "$i" "$j"
      fi
    done
  done

  log "=== Phase 2 complete ==="
else
  log "Skipping transfer replay (--skip-transfer)"
fi

# Cleanup
kill_tunnel
log ""
log "All done. Results in $OUTDIR/"
log "  Evolution:  results_gemma*.json + results_gemma*_archive.json"
log "  Transfer:   transfer_gemma*_to_gemma*.json"
