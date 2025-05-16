#!/usr/bin/env bash
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

  echo "Error: NAME environment variable must be alphanumeric and contain no spaces."
if [[ ! "${NAME}" =~ ^[a-zA-Z0-9_]+$ ]]; then
  echo "Error: NAME environment variable contains invalid characters. Only alphanumeric characters and underscores are allowed."
  exit 1
fi

if [ -z "${NAME}" ]; then
  echo "Error: NAME environment variable is not set."
  echo "Usage: NAME=tod VERSION=0.6.15 ./release.sh"
  exit 1
fi
cd target/release || { echo "Error: target/release directory does not exist. Ensure the build step completed successfully."; exit 1; }

echo "=== BUILDING RELEASE ===" &&
cargo build --release &&
echo "=== GZIPPING ===" &&
cd target/release || exit
tar -czf "$NAME-mac.tar.gz" "$NAME"
cd ../..
echo "=== CREATING GITHUB RELEASE ===" &&
gh release create "v$VERSION" ./target/release/*.tar.gz --title "v$VERSION" --generate-notes &&
echo "=== RUNNING cargo publish FOR CRATES.IO ===" &&
cargo publish || { echo "Error: cargo publish failed. Please check your credentials, network connection, or other potential issues."; exit 1; }
# echo "=== RUNNING push_aur.sh TO PUSH NEW VERSION TO AUR ===" &&
# ./scripts/push_aur.sh &&
echo "=== DELETING MERGED BRANCHES ===" &&
git-delete-merged-branches --yes &&
echo "Update Homebrew formula with the following details:" &&
if [ -f "./target/release/tod-mac.tar.gz" ]; then
  shasum -a 256 ./target/release/tod-mac.tar.gz
else
  echo "Error: File ./target/release/tod-mac.tar.gz does not exist. Ensure the tar command succeeded."
  exit 1
fi
echo "Edit the Homebrew formula at ./homebrew-tod/Formula/tod.rb with the new version and SHA sum."
