#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
args=()
for arg in "$@"; do
  case "$arg" in
    --release) export OMARCHY_BUILD_PROFILE=release ;;
    --install) args+=(--install) ;;
    *) echo 'Usage: bash scripts/package-omarchy.sh [--install] [--release]' >&2; exit 2 ;;
  esac
done
exec python3 linux/package.py "${args[@]}"
