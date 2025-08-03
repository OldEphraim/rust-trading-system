echo "Running Rust Trading System Test Suite"
echo "======================================"

echo "1. Running unit tests..."
cargo test unit --lib

echo "2. Running integration tests (mocked)..."
cargo test integration

echo "3. Running property tests..."
cargo test property

echo "4. Running performance benchmarks..."
cargo test benchmark -- --nocapture

echo "5. Running real testnet integration tests (if enabled)..."
if [ "$INTEGRATION_TESTS" = "1" ]; then
    cargo test end_to_end -- --nocapture
else
    echo "Skipped (set INTEGRATION_TESTS=1 to enable)"
fi

echo "6. Generating test coverage report..."
# cargo tarpaulin --out Html  # Uncomment if you install cargo-tarpaulin

echo "All tests completed!"
