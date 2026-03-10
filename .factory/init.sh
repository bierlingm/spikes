#!/bin/bash
set -e

# Build CLI to verify dependencies
echo "Building CLI..."
cd cli && cargo build 2>&1 | tail -5
cd ..

# Install worker dependencies
echo "Installing worker dependencies..."
cd ../spikes-hosted/worker && npm install 2>&1 | tail -3
cd -

echo "Environment ready."
