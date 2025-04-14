#!/bin/bash
# Check if the VERSION environment variable is set
if [ -z "${VERSION}" ]; then
  echo "Error: VERSION environment variable is not set."
  echo "Usage: VERSION=0.6.15 ./publish.sh"
  exit 1
fi

echo "=== VERSION IS $VERSION ===" &&
echo "=== EDITING CARGO.TOML TO NEW VERSION ===" &&
ambr --regex "^version = \"\d+\.\d+\.\d+\"" "version = \"$VERSION\"" Cargo.toml &&
echo "=== CARGO UPDATE ===" &&
cargo update &&
echo "=== FORMAT ===" &&
cargo fmt &&
echo "=== CLIPPY ===" &&
cargo clippy -- -D warnings &&
echo "=== TEST ===" &&
cargo test &&
echo "=== CREATING PR ===" &&
gt create "v$VERSION" -a -m "v$VERSION" --no-interactive &&
gt submit --no-interactive &&
gh pr ready &&
sleep 5 &&
gh pr checks --watch -i 5;
afplay /System/Library/Sounds/Ping.aiff;

