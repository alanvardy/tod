#!/bin/bash
echo "=== CARGO UPDATE ===" &&
cargo update &&
echo "=== TEST.SH ===" &&
./test.sh 
