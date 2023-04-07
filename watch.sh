#!/bin/sh
cargo watch -x check -x clippy -x test -s "rg TODO --type rust" && \
echo "SUCCESS"


