#!/bin/bash

# Integration test script for console-exporter
# This script demonstrates the console exporter functionality

echo "Running console exporter integration tests..."

# Build the project
cargo build

# Run the integration example
echo "Running integration example..."
cargo run -p console-exporter-integration

echo "Integration test completed!"