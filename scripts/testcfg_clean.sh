#!/usr/bin/env bash
# Purpose: This script deletes all test configuration files in the tests directory
# Usage: ./test_cfg_cleanup.sh

# Count the number of files matching the pattern
deleted_count=$(find ./tests/ -name "*.testcfg" | wc -l)

# If there are files to delete, delete them and print the count
if [ "$deleted_count" -gt 0 ]; then
    rm ./tests/*.testcfg
    echo "Cleaned up $deleted_count .testcfg files"
else
    echo "No files to delete"
fi