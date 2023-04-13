#!/bin/bash
grep -rE --include="*.rs" "$1" .
if [ $? -eq 0 ]; then
    echo "'$1's found."
    exit 1
else
    echo "No '$1's found."
    exit 0
fi
