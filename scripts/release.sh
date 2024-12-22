#!/bin/bash
# Publish Checklist

echo "=== RUNNING cargo aur ===" &&
cargo aur &&
echo "=== CREATING GITHUB RELEASE ===" &&
gh release create "v$VERSION" ./target/cargo-aur/*.tar.gz --title "v$VERSION" --generate-notes &&
echo "=== RUNNING cargo publish ===" &&
cargo publish &&
echo "=== RUNNING push_aur.sh ===" &&
./push_aur.sh &&
echo "=== DELETING MERGED BRANCHES ===" &&
git-delete-merged-branches --yes
