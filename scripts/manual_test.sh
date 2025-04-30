#!/usr/bin/env bash
# Does not cover the complete function
# Run it manually to ensure that we didn't break clap

commands=(
"cargo run -- -h"
"cargo run -- task -h"
"cargo run -- t q --content this is a test"
"cargo run -- task create --project '⭐  Tasks' --content \"test\" --priority 2 --description \"THIS IS DESCRIPTION\""
"cargo run -- task edit"
"cargo run -- task edit -p '⭐  Tasks'"
"cargo run -- list view"
"cargo run -- project -h"
"cargo run -- project list"
"cargo run -- project empty --project Inbox"
"cargo run -- project empty -p Inbox"
"cargo run -- list schedule --project '⭐  Tasks'"
"cargo run -- list prioritize --project '⭐  Tasks'"
"cargo run -- list prioritize -p '⭐  Tasks'"
"cargo run -- list process -p Inbox"
"cargo run -- list process --project Inbox"
"cargo run -- project import"
)

for cmd in "${commands[@]}"
do
  echo ""
  echo ""
  echo "Executing command: $cmd"

  if ! eval "$cmd"; then
    echo "Command failed: $cmd"
    exit 1
  fi
done
