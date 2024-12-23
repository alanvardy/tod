#!/bin/bash
# Publish Checklist

# Check if the VERSION environment variable is set
if [ -z "${VERSION}" ]; then
  echo "Error: VERSION environment variable is not set."
  echo "Usage: NAME=tod VERSION=0.6.15 ./release.sh"
  exit 1
fi

if [ -z "${NAME}" ]; then
  echo "Error: NAME environment variable is not set."
  echo "Usage: NAME=tod VERSION=0.6.15 ./release.sh"
  exit 1
fi

echo "=== RUNNING cargo aur TO BUILD A RELEASE ===" &&
cargo aur &&
echo "=== CREATING GITHUB RELEASE ===" &&
gh release create "v$VERSION" ./target/cargo-aur/*.tar.gz --title "v$VERSION" --generate-notes &&
echo "=== RUNNING cargo publish FOR CRATES.IO ===" &&
cargo publish &&
echo "=== RUNNING push_aur.sh TO PUSH NEW VERSION TO AUR ===" &&
./push_aur.sh &&
echo "=== DELETING MERGED BRANCHES ===" &&
git-delete-merged-branches --yes
