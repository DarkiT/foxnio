#!/bin/bash

# 运行单元测试
# Run unit tests only

set -e

echo "🔬 Running Unit Tests"
echo "================================"

cargo test --test account_service_test --lib --tests -- --nocapture
cargo test --test model_router_test --lib --tests -- --nocapture
cargo test --test rate_limit_test --lib --tests -- --nocapture
cargo test --test gateway_proxy_test --lib --tests -- --nocapture
cargo test --test gateway_handler_complete_test --lib --tests -- --nocapture

echo ""
echo "✓ Unit tests completed"
