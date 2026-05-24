#!/usr/bin/env bash
set -euo pipefail
export RUST_LOG="${RUST_LOG:-session_scheduler=debug,poise=info,serenity=info}"
cargo run
