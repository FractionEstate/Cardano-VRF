#!/bin/bash
# CI Check Script - Run all checks locally before pushing
# This script runs the same checks as CI to catch issues early

set -e  # Exit on error

echo "🔍 Running CI checks..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        exit 1
    fi
}

# 1. Check formatting
echo "📋 Checking code formatting..."
cargo fmt --check
print_status $? "Code formatting"
echo ""

# 2. Run clippy
echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
print_status $? "Clippy checks"
echo ""

# 3. Build
echo "🔨 Building project..."
cargo build --all-features
print_status $? "Build"
echo ""

# 4. Run tests
echo "🧪 Running tests..."
cargo test --all-features
print_status $? "Tests"
echo ""

# 5. Run doc tests
echo "📚 Running doc tests..."
cargo test --doc
print_status $? "Doc tests"
echo ""

# 6. Build documentation
echo "📖 Building documentation..."
cargo doc --no-deps
print_status $? "Documentation build"
echo ""

# 7. Check for security advisories
echo "🔒 Checking for security advisories..."
if command -v cargo-audit &> /dev/null; then
    cargo audit
    print_status $? "Security audit"
else
    echo -e "${YELLOW}⚠${NC}  cargo-audit not installed. Run: cargo install cargo-audit"
fi
echo ""

# Summary
echo ""
echo -e "${GREEN}✅ All CI checks passed!${NC}"
echo ""
echo "You can now safely push your changes:"
echo "  git push origin <branch-name>"
echo ""
