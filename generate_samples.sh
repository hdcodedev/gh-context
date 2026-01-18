#!/bin/bash
set -e

# Build the project first to avoid recompiling for each run
echo "Building project..."
cargo build --release

BIN_PATH="../target/release/gh-context"

# Create samples directory and enter it
mkdir -p samples
cd samples

# Issues
echo "Generating sample for (Markdown)..."
$BIN_PATH https://github.com/rust-lang/rust/issues/32838
$BIN_PATH https://github.com/brave/brave-browser/issues/5717

echo "Generating sample for (JSON)..."
$BIN_PATH https://github.com/rust-lang/rust/issues/32838 --format json
$BIN_PATH https://github.com/brave/brave-browser/issues/5717 --format json

# PRs
echo "Generating sample for (Markdown)..."
$BIN_PATH https://github.com/rust-lang/rust/pull/137330
$BIN_PATH https://github.com/home-assistant/android/pull/6237

echo "Generating sample for (JSON)..."
$BIN_PATH https://github.com/rust-lang/rust/pull/137330 --format json
$BIN_PATH https://github.com/home-assistant/android/pull/6237 --format json

echo "Sample generation complete. Check the 'samples' directory."
