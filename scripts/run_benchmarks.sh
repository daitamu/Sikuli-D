#!/bin/bash
# Benchmark execution script for sikulix-core
# sikulix-core のベンチマーク実行スクリプト

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== SikuliX Core Benchmarks ===${NC}"
echo "Starting benchmark suite..."
echo ""

# Navigate to core-rs directory
cd "$(dirname "$0")/../core-rs" || exit 1

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo not found. Please install Rust.${NC}"
    exit 1
fi

# Create results directory
RESULTS_DIR="benchmark_results"
mkdir -p "$RESULTS_DIR"

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULTS_FILE="$RESULTS_DIR/benchmark_${TIMESTAMP}.txt"

echo -e "${YELLOW}Results will be saved to: $RESULTS_FILE${NC}"
echo ""

# System information
echo "=== System Information ===" | tee -a "$RESULTS_FILE"
echo "Date: $(date)" | tee -a "$RESULTS_FILE"
echo "Rust version: $(rustc --version)" | tee -a "$RESULTS_FILE"
echo "Cargo version: $(cargo --version)" | tee -a "$RESULTS_FILE"

if command -v uname &> /dev/null; then
    echo "OS: $(uname -s)" | tee -a "$RESULTS_FILE"
    echo "Kernel: $(uname -r)" | tee -a "$RESULTS_FILE"
fi

if command -v nproc &> /dev/null; then
    echo "CPU cores: $(nproc)" | tee -a "$RESULTS_FILE"
fi

echo "" | tee -a "$RESULTS_FILE"

# Build in release mode first
echo -e "${YELLOW}Building in release mode...${NC}"
cargo build --release

echo ""
echo -e "${GREEN}Running benchmarks...${NC}"
echo ""

# Run all benchmarks
echo "=== Benchmark Results ===" | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# Matching benchmarks
echo -e "${YELLOW}[1/3] Running image matching benchmarks...${NC}"
cargo bench --bench matching 2>&1 | tee -a "$RESULTS_FILE"

echo "" | tee -a "$RESULTS_FILE"

# Screen capture benchmarks
echo -e "${YELLOW}[2/3] Running screen capture benchmarks...${NC}"
cargo bench --bench screen_capture 2>&1 | tee -a "$RESULTS_FILE"

echo "" | tee -a "$RESULTS_FILE"

# NCC calculation benchmarks
echo -e "${YELLOW}[3/3] Running NCC calculation benchmarks...${NC}"
cargo bench --bench ncc_calculation 2>&1 | tee -a "$RESULTS_FILE"

echo ""
echo -e "${GREEN}=== Benchmarks Complete ===${NC}"
echo ""
echo -e "Results saved to: ${GREEN}$RESULTS_FILE${NC}"
echo ""

# Extract summary
echo "=== Performance Summary ===" | tee -a "$RESULTS_FILE"
echo "" | tee -a "$RESULTS_FILE"

# Parse key metrics (this is a simple grep, can be enhanced)
if command -v grep &> /dev/null; then
    echo "Key Metrics:" | tee -a "$RESULTS_FILE"
    grep -E "time:.*\[.*\]" "$RESULTS_FILE" | head -20 || true
fi

echo ""
echo -e "${YELLOW}Tip:${NC} Compare results with BENCHMARK_RESULTS.md targets"
echo -e "${YELLOW}Tip:${NC} Run 'cargo bench -- --save-baseline <name>' to save a baseline for comparison"
echo ""

# Optional: Generate Criterion HTML report link
if [ -d "target/criterion" ]; then
    echo -e "${GREEN}Criterion reports available at:${NC}"
    echo "  file://$(pwd)/target/criterion/report/index.html"
    echo ""
fi

exit 0
