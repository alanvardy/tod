#!/usr/bin/env bash
# Publish Checklist

# Check if the VERSION environment variable is set
if [ -z "${VERSION}" ]; then
  echo "Error: VERSION environment variable is not set."
  echo "Usage: NAME=tod VERSION=0.6.15 ./release.sh"
  exit 1
fi


if [[ ! "${NAME}" =~ ^[a-zA-Z0-9]+$ ]]; then
  echo "Error: NAME environment variable must be alphanumeric and contain no spaces."
  exit 1
fi
if [ -z "${NAME}" ]; then
if [[ ! "${NAME}" =~ ^[a-zA-Z0-9_]+$ ]]; then
  echo "Usage: NAME=tod VERSION=0.6.15 ./release.sh"
  exit 1
fi

echo "=== BUILDING RELEASE ==="
if ! cargo build --release; then
echo "=== GZIPPING ===" &&
cd target/release || exit
if [ ! -f "tod" ]; then
  echo "Error: 'tod' binary not found in target/release. Build might have failed."
  exit 1
fi
OS_NAME=$(uname | tr '[:upper:]' '[:lower:]')
TARBALL_NAME="tod-${OS_NAME}.tar.gz"
tar -czf "$TARBALL_NAME" tod
cd ../..
fi
tar -czf tod-mac.tar.gz tod 
if [ ! -f "./target/release/tod-mac.tar.gz" ]; then
  echo "Error: 'tod-mac.tar.gz' file not found. Gzip step might have failed."
  exit 1
fi
cd ../..
echo "=== CREATING GITHUB RELEASE ===" &&
gh release create "v$VERSION" ./target/release/*.tar.gz --title "v$VERSION" --generate-notes &&
echo "=== RUNNING cargo publish FOR CRATES.IO ===" &&
cargo publish &&
# This needs to be re-implemented for the new AUR system
# echo "=== RUNNING push_aur.sh TO PUSH NEW VERSION TO AUR ===" &&
# ./scripts/push_aur.sh &&
if [ ! -f "./target/release/$TARBALL_NAME" ]; then
  echo "Error: '$TARBALL_NAME' file not found. Gzip step might have failed."
  exit 1
fi
shasum -a 256 "./target/release/$TARBALL_NAME"
  exit 1
fi
shasum -a 256 ./target/release/tod-mac.tar.gz
