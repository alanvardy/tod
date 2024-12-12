#!/bin/bash
# Check if the VERSION environment variable is set
if [ -z "${VERSION}" ]; then
  echo "Error: VERSION environment variable is not set."
  echo "Usage: VERSION=v0.6.15 ./publish.sh"
  exit 1
fi

echo "=== VERSION IS $VERSION ===" &&
echo "=== EDITING CARGO.TOML ===" &&
ambr --regex "^version = \"\d+\.\d+\.\d+\"" "version = \"$VERSION\"" Cargo.toml &&
echo "=== UPDATE AND TEST ===" &&
./update_test.sh &&
echo "=== CREATING PR ===" &&
gb "$VERSION" "$VERSION" &&
prr &&
sleep 5 &&
checks
