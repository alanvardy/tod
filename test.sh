#!/bin/bash
echo "=== FORMAT ===" &&
cargo fmt &&
echo "=== CLIPPY ===" &&
cargo clippy -- -D warnings &&
echo "=== TARPAULIN ===" &&
cargo tarpaulin -o lcov &&
echo "=== SUCCESS ==="
