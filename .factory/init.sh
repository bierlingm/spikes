#!/bin/bash
set -e

# Install CLI dependencies (build check)
echo "Building CLI..."
cd cli && cargo build 2>&1 | tail -5
cd ..

# Install Worker dependencies
echo "Installing Worker dependencies..."
cd ../spikes-hosted/worker && npm install --silent
cd ../../spikes

echo "Environment ready."
