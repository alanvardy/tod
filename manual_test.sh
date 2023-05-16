#!/bin/bash
# Does not cover the complete function
# Run it manually to ensure that we didn't break clap

commands=(
"cargo run -- -h"
"cargo run -- task -h"
"cargo run -- -q this is a test"
"cargo run -- --quickadd this is a test"
"cargo run -- task create -c \"test\""
"cargo run -- task create --content \"test\""
"cargo run -- task create -p digital -c \"test\""
"cargo run -- task create --project digital --content \"test\""
"cargo run -- task list"
"cargo run -- task list -s"
"cargo run -- task list --scheduled"
"cargo run -- project -h"
"cargo run -- project list"
"cargo run -- project add --name test --id 2"
"cargo run -- project remove --project test"
"cargo run -- project add -n test -i 2"
"cargo run -- project remove -p test"
"cargo run -- project empty --project inbox"
"cargo run -- project empty -p inbox"
"cargo run -- project schedule --project digital"
"cargo run -- project schedule -p physical"
"cargo run -- project prioritize --project digital"
"cargo run -- project prioritize -p physical"
"cargo run -- project process -p inbox"
"cargo run -- project process --project inbox"
)

for cmd in "${commands[@]}"
do
  echo "Executing command: $cmd"

  if ! eval "$cmd"; then
    echo "Command failed: $cmd"
    exit 1
  fi
done