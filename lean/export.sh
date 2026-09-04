#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
lake build
lake env lean XLemma/Example.lean
