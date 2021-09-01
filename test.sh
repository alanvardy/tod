#!/bin/sh
cargo fmt &&
cargo clippy &&
cargo tarpaulin -o lcov &&
echo "SUCCESS"
