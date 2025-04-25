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
echo "=== SUCCESS ==="
