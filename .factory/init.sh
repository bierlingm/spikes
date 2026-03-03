#!/bin/bash
set -e

# Build CLI to verify dependencies
echo "Building CLI..."
cd cli && cargo build 2>&1 | tail -5
cd ..

echo "Environment ready."
