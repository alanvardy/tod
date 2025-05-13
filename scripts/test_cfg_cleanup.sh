#!/usr/bin/env bash
# Purpose: This script deletes all test configuration files in the tests directory
# Usage: ./test_cfg_cleanup.sh

# Count the number of files to be deleted and delete them
deleted_files=$(rm ./tests/*.testcfg)

# Check if any files were deleted
deleted_count=$(echo "$deleted_files" | wc -l)

# Output the number of deleted files
echo "Deleted $deleted_count files"