#!/bin/bash
# Run All AMP Tests

echo "=== AMP Test Suite ==="
echo ""

BASE_URL="http://localhost:8105"

# Check if server is running
echo "Checking server status..."
if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
    echo "✓ Server is running"
    echo ""
else
    echo "✗ Server is not running on $BASE_URL"
    echo "Please start the server first: cd server && cargo run"
    exit 1
fi

PASSED=0
FAILED=0

run_test() {
    local name="$1"
    local script="$2"
    
    echo "Running: $name"
    echo "============================================================"
    
    if bash "$script"; then
        ((PASSED++))
        echo ""
        echo "✓ $name PASSED"
    else
        ((FAILED++))
        echo ""
        echo "✗ $name FAILED"
    fi
    
    echo ""
    echo ""
}

run_test "CRUD Operations" "test-crud.sh"
run_test "Schema Validation" "test-schemas.sh"
run_test "Demo Workflow" "demo.sh"

echo "============================================================"
echo "Test Summary"
echo "============================================================"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "All tests passed! ✓"
    exit 0
else
    echo "Some tests failed. Please review the output above."
    exit 1
fi
