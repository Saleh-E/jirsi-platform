#!/bin/bash
# Load Testing Script using wrk

echo "🔥 Jirsi Platform - Load Testing"
echo "=================================="

# Configuration
TARGET_URL="http://localhost:8080"
DURATION="30s"
THREADS=4
CONNECTIONS=100

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
    echo "❌ wrk not found. Installing..."
    # macOS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install wrk
    # Linux
    elif command -v apt-get &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y wrk
    else
        echo "Please install wrk manually: https://github.com/wg/wrk"
        exit 1
    fi
fi

echo -e "${GREEN}✅ wrk found${NC}"
echo ""

# Test 1: Sync Status Endpoint (Health Check)
echo -e "${YELLOW}Test 1: Sync Status Endpoint${NC}"
wrk -t${THREADS} -c${CONNECTIONS} -d${DURATION} \
    ${TARGET_URL}/api/v1/sync/status

echo ""
echo "---"
echo ""

# Test 2: Entity List (Database Query)
echo -e "${YELLOW}Test 2: Entity List Endpoint${NC}"
wrk -t${THREADS} -c${CONNECTIONS} -d${DURATION} \
    -H "Authorization: Bearer test-token" \
    ${TARGET_URL}/api/v1/entities/contact

echo ""
echo "---"
echo ""

# Test 3: Metrics Endpoint
echo -e "${YELLOW}Test 3: Metrics Endpoint${NC}"
wrk -t${THREADS} -c${CONNECTIONS} -d${DURATION} \
    ${TARGET_URL}/metrics

echo ""
echo "---"
echo ""

# Performance Summary
echo -e "${GREEN}📊 Performance Summary${NC}"
echo "=================================="
echo "Target Metrics:"
echo "  - Throughput: > 1000 req/sec"
echo "  - Latency p95: < 100ms"
echo "  - Latency p99: < 500ms"
echo ""
echo "If metrics don't meet targets:"
echo "  1. Check database indexes"
echo "  2. Verify cache hit rate"
echo "  3. Review connection pool settings"
echo "  4. Analyze slow queries"
