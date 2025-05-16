#!/usr/bin/env bash
echo "=== CHECK ===" &&
cargo check &&
echo "=== CLIPPY ===" &&
cargo clippy -- -D warnings &&
echo "=== TEST ===" &&
cargo nextest run &&
echo "=== TODOS ===" &&
./scripts/lint_string.sh "TODO " &&
./scripts/lint_string.sh "TODO:" &&
./scripts/lint_string.sh "FIXME " &&
./scripts/lint_string.sh "FIXME:" &&
./scripts/lint_string.sh "todo " &&
./scripts/lint_string.sh "todo:" &&
./scripts/lint_string.sh "fixme " &&
./scripts/lint_string.sh "fixme:" &&
./scripts/lint_string.sh "dbg!" &&
./scripts/lint_string.sh "DEBUG:" &&
./scripts/lint_string.sh "FIXTURE:" &&
echo "=== SUCCESS ===" &&
echo "=== CLEANING FILES ===" &&
./scripts/testcfg_clean.sh &&
echo "=== Done ===."
