#!/bin/bash
cargo fmt &&
cargo clippy -- -D warnings &&
cargo tarpaulin -o lcov &&
echo "SUCCESS"
