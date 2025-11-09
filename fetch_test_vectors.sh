#!/bin/bash
# fetch_test_vectors.sh
#
# This script fetches all official VRF test vectors from the cardano-base-rust repository
# and saves them to a local directory for integration into our test suite.

set -e

# Configuration
REPO_URL="https://raw.githubusercontent.com/FractionEstate/cardano-base-rust/main"
VECTORS_PATH="cardano-test-vectors/test_vectors"
OUTPUT_DIR="./test_vectors"

# Color output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "╔════════════════════════════════════════════════════════════╗"
echo "║     Cardano VRF Test Vector Fetcher                       ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# List of all official test vectors
DRAFT03_STANDARD="vrf_ver03_standard_10 vrf_ver03_standard_11 vrf_ver03_standard_12"
DRAFT03_GENERATED="vrf_ver03_generated_1 vrf_ver03_generated_2 vrf_ver03_generated_3 vrf_ver03_generated_4"
DRAFT13_STANDARD="vrf_ver13_standard_10 vrf_ver13_standard_11 vrf_ver13_standard_12"
DRAFT13_GENERATED="vrf_ver13_generated_1 vrf_ver13_generated_2 vrf_ver13_generated_3 vrf_ver13_generated_4"

ALL_VECTORS="$DRAFT03_STANDARD $DRAFT03_GENERATED $DRAFT13_STANDARD $DRAFT13_GENERATED"

total=0
success=0
failed=0

echo "Fetching test vectors from:"
echo "  Repository: FractionEstate/cardano-base-rust"
echo "  Path: $VECTORS_PATH"
echo ""

for vector in $ALL_VECTORS; do
    total=$((total + 1))
    url="$REPO_URL/$VECTORS_PATH/$vector"
    output="$OUTPUT_DIR/$vector"

    printf "Fetching %-30s ... " "$vector"

    if curl -sf "$url" -o "$output"; then
        echo -e "${GREEN}✓${NC}"
        success=$((success + 1))
    else
        echo -e "${RED}✗${NC}"
        failed=$((failed + 1))
    fi
done

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                     FETCH SUMMARY                          ║"
echo "╠════════════════════════════════════════════════════════════╣"
printf "║  Total Vectors:      %-34s║\n" "$total"
printf "║  Successfully Fetched: ${GREEN}%-31s${NC}║\n" "$success"
printf "║  Failed:            ${RED}%-34s${NC}║\n" "$failed"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

if [ $success -eq $total ]; then
    echo -e "${GREEN}✅ All test vectors fetched successfully!${NC}"
    echo ""
    echo "Test vectors saved to: $OUTPUT_DIR/"
    echo ""
    echo "Next steps:"
    echo "  1. Review the fetched vectors: ls -l $OUTPUT_DIR/"
    echo "  2. Parse and integrate into tests/official_test_vectors.rs"
    echo "  3. Run: cargo test --test official_test_vectors"
    echo ""
    echo "Example vector format:"
    echo "  cat $OUTPUT_DIR/vrf_ver03_standard_10"
else
    echo -e "${YELLOW}⚠️  Some vectors failed to fetch.${NC}"
    echo "Please check your internet connection and try again."
    exit 1
fi
