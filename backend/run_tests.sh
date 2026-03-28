#!/bin/bash

# FoxNIO Backend Test Runner
# 运行所有测试并生成覆盖率报告

set -e

echo "🧪 FoxNIO Backend Test Runner"
echo "================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 函数：运行测试套件
run_test_suite() {
    local suite_name=$1
    local test_path=$2
    
    echo -e "${YELLOW}Running $suite_name...${NC}"
    
    if cargo test --test "$test_path" --lib --tests 2>&1 | tee /tmp/test_output.txt; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo -e "${GREEN}✓ $suite_name passed${NC}"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "${RED}✗ $suite_name failed${NC}"
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo ""
}

# 检查 cargo 是否安装
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo is not installed${NC}"
    echo "Please install Rust and cargo first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# 检查项目目录
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found${NC}"
    echo "Please run this script from the project root directory"
    exit 1
fi

echo "📦 Building project..."
cargo build --release
echo ""

echo "🧹 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
echo ""

echo "🔬 Running unit tests..."
echo "================================"

# 单元测试
run_test_suite "Account Service Tests" "account_service_test"
run_test_suite "Model Router Tests" "model_router_test"
run_test_suite "Rate Limit Tests" "rate_limit_test"
run_test_suite "Gateway Proxy Tests" "gateway_proxy_test"
run_test_suite "Gateway Handler Tests" "gateway_handler_complete_test"

echo ""
echo "🔬 Running integration tests..."
echo "================================"

# 集成测试
run_test_suite "Alert Tests" "alert_test"
run_test_suite "Scheduler Tests" "scheduler_test"
run_test_suite "Failover Tests" "failover_test"
run_test_suite "Compression Tests" "compression_test"
run_test_suite "Concurrency Tests" "concurrency_test"
run_test_suite "Health Tests" "health_test"
run_test_suite "Metrics Tests" "metrics_test"
run_test_suite "Encryption Tests" "encryption_test"
run_test_suite "JWT Refresh Tests" "jwt_refresh_test"
run_test_suite "Database Tests" "database_test"
run_test_suite "Redis Tests" "redis_test"

echo ""
echo "📊 Test Summary"
echo "================================"
echo -e "Total Test Suites: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
