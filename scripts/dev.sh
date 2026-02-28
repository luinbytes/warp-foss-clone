#!/bin/bash
# Development workflow helper for warp-foss-clone
# Common development tasks

set -e

COMMAND=${1:-help}

case $COMMAND in
    build)
        echo "Building release binary..."
        cargo build --release
        echo "✓ Build complete: target/release/warp-foss"
        ;;
    
    run)
        echo "Running terminal..."
        cargo run --release
        ;;
    
    test)
        echo "Running tests..."
        cargo test
        ;;
    
    lint)
        echo "Running linter..."
        cargo clippy -- -D warnings
        ;;
    
    fmt)
        echo "Formatting code..."
        cargo fmt
        echo "✓ Code formatted"
        ;;
    
    check)
        echo "Running all checks..."
        echo "  Formatting..."
        cargo fmt -- --check
        echo "  Linting..."
        cargo clippy -- -D warnings
        echo "  Testing..."
        cargo test
        echo "✓ All checks passed"
        ;;
    
    clean)
        echo "Cleaning build artifacts..."
        cargo clean
        echo "✓ Clean complete"
        ;;
    
    deps)
        echo "Updating dependencies..."
        cargo update
        echo "✓ Dependencies updated"
        ;;
    
    doc)
        echo "Building documentation..."
        cargo doc --no-deps --open
        ;;
    
    release)
        echo "Preparing release..."
        ./scripts/check.sh || { echo "❌ Checks failed"; exit 1; }
        cargo build --release
        strip target/release/warp-foss
        echo "✓ Release binary ready: target/release/warp-foss"
        ;;
    
    benchmark)
        ./scripts/benchmark.sh
        ;;
    
    profile)
        ./scripts/profile.sh
        ;;
    
    watch)
        echo "Watching for changes..."
        cargo watch -x "build" -x "test"
        ;;
    
    help|*)
        echo "Warp FOSS Clone - Development Helper"
        echo ""
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  build       Build release binary"
        echo "  run         Run the terminal"
        echo "  test        Run tests"
        echo "  lint        Run clippy linter"
        echo "  fmt         Format code"
        echo "  check       Run all checks (fmt, lint, test)"
        echo "  clean       Remove build artifacts"
        echo "  deps        Update dependencies"
        echo "  doc         Build and open documentation"
        echo "  release     Prepare release binary"
        echo "  benchmark   Run performance benchmarks"
        echo "  profile     Run profiling tools"
        echo "  watch       Watch for changes and rebuild"
        echo "  help        Show this help message"
        ;;
esac
