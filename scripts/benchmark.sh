#!/bin/bash
# Performance benchmark script for warp-foss-clone
# Measures build time, binary size, and basic operations

set -e

echo "═══════════════════════════════════════════════════════"
echo "   Warp FOSS Clone - Performance Benchmark"
echo "═══════════════════════════════════════════════════════"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to format bytes
format_bytes() {
    local bytes=$1
    if [ $bytes -gt 1073741824 ]; then
        echo "$(echo "scale=2; $bytes / 1073741824" | bc) GB"
    elif [ $bytes -gt 1048576 ]; then
        echo "$(echo "scale=2; $bytes / 1048576" | bc) MB"
    elif [ $bytes -gt 1024 ]; then
        echo "$(echo "scale=2; $bytes / 1024" | bc) KB"
    else
        echo "$bytes bytes"
    fi
}

# Function to format duration
format_duration() {
    local seconds=$1
    if [ $(echo "$seconds > 60" | bc) -eq 1 ]; then
        local mins=$(echo "$seconds / 60" | bc)
        local secs=$(echo "$seconds % 60" | bc)
        echo "${mins}m ${secs}s"
    else
        echo "${seconds}s"
    fi
}

# Build benchmark
echo -e "${YELLOW}1. Build Performance${NC}"
echo "───────────────────────────────────────────────────────"

echo -n "   Clean build time: "
START=$(date +%s.%N)
cargo clean > /dev/null 2>&1
cargo build --release > /dev/null 2>&1
END=$(date +%s.%N)
BUILD_TIME=$(echo "$END - $START" | bc)
echo -e "${GREEN}$(format_duration $BUILD_TIME)${NC}"

echo -n "   Incremental build time: "
START=$(date +%s.%N)
cargo build --release > /dev/null 2>&1
END=$(date +%s.%N)
INC_BUILD_TIME=$(echo "$END - $START" | bc)
echo -e "${GREEN}$(format_duration $INC_BUILD_TIME)${NC}"

echo ""

# Binary size
echo -e "${YELLOW}2. Binary Metrics${NC}"
echo "───────────────────────────────────────────────────────"

BINARY_PATH="target/release/warp-foss"
if [ -f "$BINARY_PATH" ]; then
    BINARY_SIZE=$(stat -f%z "$BINARY_PATH" 2>/dev/null || stat -c%s "$BINARY_PATH" 2>/dev/null)
    echo -n "   Binary size: "
    echo -e "${GREEN}$(format_bytes $BINARY_SIZE)${NC}"
    
    echo -n "   Stripped size: "
    strip "$BINARY_PATH" -o /tmp/warp-foss-stripped > /dev/null 2>&1
    STRIPPED_SIZE=$(stat -f%z /tmp/warp-foss-stripped 2>/dev/null || stat -c%s /tmp/warp-foss-stripped 2>/dev/null)
    echo -e "${GREEN}$(format_bytes $STRIPPED_SIZE)${NC}"
    rm /tmp/warp-foss-stripped
else
    echo -e "   ${RED}Binary not found at $BINARY_PATH${NC}"
fi

echo ""

# Dependency analysis
echo -e "${YELLOW}3. Dependency Analysis${NC}"
echo "───────────────────────────────────────────────────────"

echo -n "   Total dependencies: "
DEP_COUNT=$(cargo tree --prefix none 2>/dev/null | wc -l | tr -d ' ')
echo -e "${GREEN}$DEP_COUNT${NC}"

echo -n "   Direct dependencies: "
DIRECT_DEPS=$(cargo tree --depth 1 --prefix none 2>/dev/null | tail -n +2 | wc -l | tr -d ' ')
echo -e "${GREEN}$DIRECT_DEPS${NC}"

echo ""

# Code metrics
echo -e "${YELLOW}4. Code Metrics${NC}"
echo "───────────────────────────────────────────────────────"

echo -n "   Total lines of code: "
LOC=$(find src -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')
echo -e "${GREEN}$LOC${NC}"

echo -n "   Source files: "
FILE_COUNT=$(find src -name "*.rs" | wc -l | tr -d ' ')
echo -e "${GREEN}$FILE_COUNT${NC}"

echo ""

# Test metrics
echo -e "${YELLOW}5. Test Coverage${NC}"
echo "───────────────────────────────────────────────────────"

echo -n "   Running tests: "
TEST_OUTPUT=$(cargo test --release 2>&1)
TEST_PASSED=$(echo "$TEST_OUTPUT" | grep -o "[0-9]\+ passed" | head -1 | grep -o "[0-9]\+" || echo "0")
TEST_FAILED=$(echo "$TEST_OUTPUT" | grep -o "[0-9]\+ failed" | head -1 | grep -o "[0-9]\+" || echo "0")

if [ "$TEST_FAILED" -eq 0 ]; then
    echo -e "${GREEN}$TEST_PASSED tests passed${NC}"
else
    echo -e "${RED}$TEST_PASSED passed, $TEST_FAILED failed${NC}"
fi

echo ""

# Compilation warnings
echo -e "${YELLOW}6. Code Quality${NC}"
echo "───────────────────────────────────────────────────────"

echo "   Checking for warnings..."
WARNING_COUNT=$(cargo clippy --release 2>&1 | grep -c "warning:" || echo "0")
if [ "$WARNING_COUNT" -eq 0 ]; then
    echo -e "   ${GREEN}No warnings found ✓${NC}"
else
    echo -e "   ${YELLOW}$WARNING_COUNT warnings found${NC}"
fi

echo ""
echo "═══════════════════════════════════════════════════════"
echo "   Benchmark Complete"
echo "═══════════════════════════════════════════════════════"
