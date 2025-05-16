#!/usr/bin/env bash
# Purpose: This script checks for the presence of "fixme" and similar text within the rust code to ensure that no such text is present in the codebase.
# Usage: ./lint_string.sh <string>
matches=$(find . -name "*.rs" -print0 | xargs -0 grep -E "$1" | wc -l)
if [ "$matches" -gt 0 ]; then
    echo "String '$1' was found $matches times in the Rust codebase."
    exit 1
else
    echo "No occurrences of string '$1' were found in the Rust codebase."
    exit 0
fi
