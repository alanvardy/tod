#!/bin/bash
# Publish Checklist

echo "=== RUNNING cargo aur ===" &&
cargo aur &&
echo "=== RUNNING cargo publish ===" &&
cargo publish &&
echo "=== RUNNING push_aur.sh ===" &&
./push_aur.sh &&
echo "=== DELETING MERGED BRANCHES ===" &&
git-delete-merged-branches --yes
