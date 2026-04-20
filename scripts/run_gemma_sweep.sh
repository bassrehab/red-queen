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

# Pod 1: Target (Gemma models)
TARGET_HOST="root@103.196.86.188"
TARGET_PORT="28788"
TARGET_SSH="-i $SSH_KEY -o StrictHostKeyChecking=no -o ConnectTimeout=30 -p $TARGET_PORT"

# Pod 2: Judge (Llama-3.1-8B-Instruct)
JUDGE_HOST="root@203.57.40.214"
JUDGE_PORT="10249"
JUDGE_SSH="-i $SSH_KEY -o StrictHostKeyChecking=no -o ConnectTimeout=30 -p $JUDGE_PORT"

VLLM_PORT=8000
LOCAL_TUNNEL_PORT=8000
LOCAL_JUDGE_PORT=8001

# Unified self-hosted judge — no OpenAI API dependency
JUDGE="openai-compat:meta-llama/Llama-3.1-8B-Instruct@http://localhost:${LOCAL_JUDGE_PORT}"
SEEDS="experiments/seeds/harmbench.jsonl"
OUTDIR="results"

# MAP-Elites config
ME_BINS=6
ME_BATCH=50
GENERATIONS=200
CONCURRENCY=5

# Reproducibility seeds (3 seeds per model)
RNG_SEEDS=(42 1337 2718)

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

ssh_target() { ssh $TARGET_SSH "$TARGET_HOST" "$@"; }
ssh_judge()  { ssh $JUDGE_SSH "$JUDGE_HOST" "$@"; }

kill_tunnels() {
  lsof -ti tcp:$LOCAL_TUNNEL_PORT 2>/dev/null | xargs kill -9 2>/dev/null || true
  lsof -ti tcp:$LOCAL_JUDGE_PORT 2>/dev/null | xargs kill -9 2>/dev/null || true
}

start_tunnels() {
  kill_tunnels
  ssh $TARGET_SSH -o ServerAliveInterval=60 -o ServerAliveCountMax=5 \
    -o ExitOnForwardFailure=yes \
    -L ${LOCAL_TUNNEL_PORT}:localhost:${VLLM_PORT} -fN "$TARGET_HOST"
  ssh $JUDGE_SSH -o ServerAliveInterval=60 -o ServerAliveCountMax=5 \
    -o ExitOnForwardFailure=yes \
    -L ${LOCAL_JUDGE_PORT}:localhost:${VLLM_PORT} -fN "$JUDGE_HOST"
  log "Tunnels active: target=localhost:$LOCAL_TUNNEL_PORT, judge=localhost:$LOCAL_JUDGE_PORT"
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

  # Kill existing vLLM + orphaned EngineCore (holds GPU memory after pkill)
  ssh_target 'pkill -f "vllm.entrypoints" 2>/dev/null || true; sleep 3; pkill -9 -f "vllm.entrypoints" 2>/dev/null || true; sleep 2; nvidia-smi --query-compute-apps=pid --format=csv,noheader 2>/dev/null | while read pid; do kill -9 $pid 2>/dev/null; done; sleep 2'

  # Start new model
  ssh_target "nohup /workspace/serve.sh '$model' $VLLM_PORT > /dev/null 2>&1 & disown; echo 'launched'"

  # Ensure tunnel is alive (model swap may have disrupted it)
  start_tunnels

  # Wait for server
  wait_for_vllm "$model"
}

# --- Phase 1: Evolution runs -------------------------------------------------

run_evolution() {
  local model="$1"
  local name="$2"
  local rng_seed="$3"
  local seed_dir="$OUTDIR/${name}/seed_${rng_seed}"
  local output="${seed_dir}/results.json"

  mkdir -p "$seed_dir"

  if [ -f "$output" ]; then
    log "SKIP: $output already exists"
    return 0
  fi

  log "=== Evolution: $model seed=$rng_seed → $output ==="

  cargo run --release --quiet -- llm jailbreak \
    --target "openai-compat:${model}@http://localhost:${LOCAL_TUNNEL_PORT}" \
    --seeds "$SEEDS" \
    --judge "$JUDGE" \
    --seed "$rng_seed" \
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
  local rng_seed="$3"
  local src_name="${MODEL_NAMES[$src_idx]}"
  local dst_name="${MODEL_NAMES[$dst_idx]}"
  local dst_model="${MODELS[$dst_idx]}"
  local archive="$OUTDIR/${src_name}/seed_${rng_seed}/results_archive.json"
  local out_dir="$OUTDIR/transfer"
  local output="${out_dir}/${src_name}_seed${rng_seed}_on_${dst_name}.json"

  mkdir -p "$out_dir"

  if [ -f "$output" ]; then
    log "SKIP: $output already exists"
    return 0
  fi

  if [ ! -f "$archive" ]; then
    log "SKIP: archive $archive not found"
    return 0
  fi

  log "=== Transfer: $src_name seed=$rng_seed → $dst_name ==="

  cargo run --release --quiet -- llm transfer \
    --archive "$archive" \
    --target "openai-compat:${dst_model}@http://localhost:${LOCAL_TUNNEL_PORT}" \
    --judge "$JUDGE" \
    --concurrency $CONCURRENCY \
    --output "$output"

  log "DONE: $output"
}

# --- Main ---------------------------------------------------------------------

log "Red Queen — Gemma Generational Sweep (multi-seed)"
log "Models: ${MODELS[*]}"
log "Seeds:  ${RNG_SEEDS[*]}"
log "Config: $GENERATIONS generations, batch $ME_BATCH, bins $ME_BINS"
log "Judge:  $JUDGE"
log ""

mkdir -p "$OUTDIR"

# Ensure tunnels are up
start_tunnels

if [ "$SKIP_EVOLUTION" = false ]; then
  log "=== Phase 1: Evolution runs (${#MODELS[@]} models × ${#RNG_SEEDS[@]} seeds) ==="
  for i in "${!MODELS[@]}"; do
    # Swap model once, run all seeds sequentially on it
    swap_model "${MODELS[$i]}"
    for s in "${RNG_SEEDS[@]}"; do
      run_evolution "${MODELS[$i]}" "${MODEL_NAMES[$i]}" "$s"
    done
    # Sanity check: all seeds should fill > 100 cells
    for s in "${RNG_SEEDS[@]}"; do
      archive="$OUTDIR/${MODEL_NAMES[$i]}/seed_${s}/results_archive.json"
      if [ -f "$archive" ]; then
        cells=$(python3 -c "import json; d=json.load(open('$archive')); print(d['filled_cells'])" 2>/dev/null || echo 0)
        if [ "$cells" -lt 100 ]; then
          log "WARNING: ${MODEL_NAMES[$i]} seed=$s has only $cells cells (< 100)"
        else
          log "OK: ${MODEL_NAMES[$i]} seed=$s has $cells cells"
        fi
      fi
    done
  done
  log "=== Phase 1 complete ==="
else
  log "Skipping evolution (--skip-evolution)"
fi

if [ "$SKIP_TRANSFER" = false ]; then
  log "=== Phase 2: Transfer replay ==="

  # For each target model j, load it once, then replay all (source, seed) archives
  for j in "${!MODELS[@]}"; do
    swap_model "${MODELS[$j]}"

    for i in "${!MODELS[@]}"; do
      if [ "$i" -ne "$j" ]; then
        for s in "${RNG_SEEDS[@]}"; do
          run_transfer "$i" "$j" "$s"
        done
      fi
    done
  done

  log "=== Phase 2 complete ==="
else
  log "Skipping transfer replay (--skip-transfer)"
fi

# Cleanup
kill_tunnels
log ""
log "All done. Results in $OUTDIR/"
log "  Evolution:  $OUTDIR/<model>/seed_<N>/results.json + results_archive.json"
log "  Transfer:   $OUTDIR/transfer/<source>_seed<N>_on_<target>.json"
