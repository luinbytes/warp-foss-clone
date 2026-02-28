#!/bin/bash
# Profiling helper script for warp-foss-clone
# Helps identify performance bottlenecks

set -e

echo "═══════════════════════════════════════════════════════"
echo "   Warp FOSS Clone - Profiling Helper"
echo "═══════════════════════════════════════════════════════"
echo ""

# Check for profiling tools
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 not found. Install with:"
        echo "   $2"
        return 1
    else
        echo "✓ $1 found"
        return 0
    fi
}

echo "Checking profiling tools..."
echo "───────────────────────────────────────────────────────"

# Check for various profiling tools
PERF_OK=0
FLAMEGRAPH_OK=0
VALGRIND_OK=0

if check_tool "perf" "sudo apt install linux-perf"; then
    PERF_OK=1
fi

if check_tool "flamegraph.pl" "cargo install flamegraph"; then
    FLAMEGRAPH_OK=1
fi

if check_tool "valgrind" "sudo apt install valgrind"; then
    VALGRIND_OK=1
fi

echo ""

# Build with debug symbols
echo "Building with debug symbols..."
echo "───────────────────────────────────────────────────────"
cargo build --profile profiling 2>&1 | grep -E "(Compiling|Finished)" || cargo build --release
echo ""

# Menu
while true; do
    echo "═══════════════════════════════════════════════════════"
    echo "   Select profiling method:"
    echo "═══════════════════════════════════════════════════════"
    echo "1) CPU profiling with perf + flamegraph"
    echo "2) Memory profiling with valgrind"
    echo "3) Heap profiling with valgrind massif"
    echo "4) Call graph with cargo flamegraph"
    echo "5) Custom command"
    echo "6) Exit"
    echo ""
    read -p "Choice [1-6]: " choice

    case $choice in
        1)
            if [ $PERF_OK -eq 0 ]; then
                echo "perf not available"
                continue
            fi
            echo ""
            echo "Running CPU profiling..."
            echo "Use the terminal for ~30 seconds, then close it"
            echo ""
            sudo perf record -g --target/release/warp-foss
            echo ""
            echo "Generating flamegraph..."
            sudo perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
            echo "✓ Flamegraph saved to: flamegraph.svg"
            ;;
        2)
            if [ $VALGRIND_OK -eq 0 ]; then
                echo "valgrind not available"
                continue
            fi
            echo ""
            echo "Running memory profiling..."
            echo "This will be slow. Use the terminal briefly, then close it"
            echo ""
            valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --verbose --log-file=valgrind-out.txt ./target/release/warp-foss
            echo "✓ Memory report saved to: valgrind-out.txt"
            ;;
        3)
            if [ $VALGRIND_OK -eq 0 ]; then
                echo "valgrind not available"
                continue
            fi
            echo ""
            echo "Running heap profiling..."
            echo "Use the terminal for a bit, then close it"
            echo ""
            valgrind --tool=massif --massif-out-file=massif.out ./target/release/warp-foss
            ms_print massif.out > heap-usage.txt
            echo "✓ Heap usage saved to: heap-usage.txt"
            ;;
        4)
            if [ $FLAMEGRAPH_OK -eq 0 ]; then
                echo "flamegraph not available"
                continue
            fi
            echo ""
            echo "Generating call graph flamegraph..."
            echo "Use the terminal for ~30 seconds, then close it"
            echo ""
            cargo flamegraph --root --output=callgraph.svg
            echo "✓ Call graph saved to: callgraph.svg"
            ;;
        5)
            read -p "Enter custom command: " cmd
            echo ""
            echo "Running: $cmd"
            eval "$cmd"
            ;;
        6)
            echo "Exiting..."
            exit 0
            ;;
        *)
            echo "Invalid choice"
            ;;
    esac
    echo ""
done
