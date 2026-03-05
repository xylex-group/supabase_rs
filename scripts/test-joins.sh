#!/usr/bin/env bash
# Run join integration tests: reset DB, apply migrations, seed, then run select_joins tests.
# Requires: Supabase CLI, Docker. Creates .env from local Supabase if missing.
#
# Run `supabase start` in another terminal first and wait until it's ready.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DB_RESET_TIMEOUT="${DB_RESET_TIMEOUT:-120}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

die() {
  echo -e "${RED}Error: $*${NC}" >&2
  exit 1
}

warn() {
  echo -e "${YELLOW}Warning: $*${NC}" >&2
}

cd "$PROJECT_ROOT"

echo "Join integration test runner"
echo "============================"

# Check required tools
command -v supabase >/dev/null 2>&1 || die "Supabase CLI not found. Install it: https://supabase.com/docs/guides/cli"
echo "  supabase CLI: OK"
command -v cargo >/dev/null 2>&1 || die "Cargo not found. Install Rust: https://rustup.rs"
echo "  cargo: OK"

# Docker required for local Supabase
if ! docker info >/dev/null 2>&1; then
  die "Docker is not running. Start Docker Desktop (or docker daemon) and retry."
fi
echo "  Docker: OK"

# Check we're in a Supabase project
if [[ ! -d supabase/migrations ]]; then
  die "Not a Supabase project: supabase/migrations not found. Run 'supabase init' first."
fi

# Supabase must already be running
echo "Checking Supabase status..."
if ! supabase status >/dev/null 2>&1; then
  die "Supabase is not running. In another terminal run: supabase start\nWait until it's ready, then run this script again."
fi
echo "  Supabase: OK"

# Create .env from local Supabase if missing or incomplete
if [[ ! -f .env ]] || ! grep -qE '^SUPABASE_URL=' .env 2>/dev/null || ! grep -qE '^SUPABASE_KEY=' .env 2>/dev/null; then
  echo "  Creating .env from local Supabase..."
  supabase status -o env 2>/dev/null | sed -e 's/^API_URL=/SUPABASE_URL=/' -e 's/^ANON_KEY=/SUPABASE_KEY=/' | grep -E '^(SUPABASE_URL|SUPABASE_KEY)=' > .env
  if [[ ! -s .env ]]; then
    die "Failed to create .env from supabase status. Run: supabase status -o env"
  fi
  echo "  .env created."
fi

# Load .env
set -a
# shellcheck source=/dev/null
source .env 2>/dev/null || true
set +a
[[ -z "${SUPABASE_URL:-}" ]] || [[ -z "${SUPABASE_KEY:-}" ]] && die ".env missing SUPABASE_URL or SUPABASE_KEY"

# Run db reset in foreground for immediate output; trap so Ctrl+C kills everything
run_db_reset() {
  local timeout_sec=$1
  local supabase_cmd=(supabase db reset --local)
  [[ "${SUPABASE_DEBUG:-0}" == "1" ]] && supabase_cmd=(supabase --debug db reset --local)

  # Progress ticker: prints every 8s so user knows it's still working
  ( while true; do sleep 8; echo "  ... still resetting ($(date +%H:%M:%S))" >&2; done ) &
  local ticker_pid=$!
  trap "kill $ticker_pid 2>/dev/null; kill -TERM -$$ 2>/dev/null; exit 130" INT TERM

  local ret=0
  if command -v timeout >/dev/null 2>&1; then
    timeout "$timeout_sec" "${supabase_cmd[@]}" || ret=$?
  elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$timeout_sec" "${supabase_cmd[@]}" || ret=$?
  else
    perl -e 'alarm shift; exec @ARGV' "$timeout_sec" "${supabase_cmd[@]}" || ret=$?
  fi

  kill $ticker_pid 2>/dev/null
  trap - INT TERM
  return $ret
}

echo ""
echo "Resetting Supabase database (timeout: ${DB_RESET_TIMEOUT}s, Ctrl+C to cancel)..."
if ! run_db_reset "$DB_RESET_TIMEOUT"; then
  exit_code=$?
  if [[ $exit_code -eq 124 ]] || [[ $exit_code -eq 143 ]] || [[ $exit_code -eq 142 ]] || [[ $exit_code -eq 130 ]]; then
    [[ $exit_code -eq 130 ]] && echo "Interrupted by user."
    [[ $exit_code -ne 130 ]] && die "supabase db reset timed out after ${DB_RESET_TIMEOUT}s. Try: DB_RESET_TIMEOUT=300 $0"
    exit 130
  fi
  die "supabase db reset failed (exit $exit_code). Run with --debug: supabase db reset --local --debug"
fi

echo "Running join integration tests..."
if ! cargo test select_joins -- --ignored; then
  die "Join tests failed."
fi

echo -e "${GREEN}Done. All join tests passed.${NC}"
