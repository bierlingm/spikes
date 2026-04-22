#!/bin/bash
set -e

# Build CLI to verify dependencies
echo "Building CLI..."
cd cli && cargo build 2>&1 | tail -5
cd ..

# Install worker dependencies (if hosted repo exists)
if [ -d "../spikes-hosted/worker" ]; then
  echo "Installing worker dependencies..."
  cd ../spikes-hosted/worker && npm install 2>&1 | tail -3
  cd -
fi

# Create test page directory for widget testing
mkdir -p /tmp/spikes-widget-test

echo "Environment ready."
