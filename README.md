# Tod

[![Build Status](https://github.com/alanvardy/tod/workflows/ci/badge.svg)](https://github.com/alanvardy/tod) [![codecov](https://codecov.io/gh/alanvardy/tod/branch/main/graph/badge.svg?token=9FBJK1SU0K)](https://codecov.io/gh/alanvardy/tod) [![Crates.io](https://img.shields.io/crates/v/tod.svg)](https://crates.io/crates/tod)

A tiny todoist CLI program. Takes simple input and dumps it in your inbox or another project. Takes advantage of natural language processing to assign due dates, tags, etc.

![Tod](tod.gif)

## Table of Contents


<!--toc:start-->
- [Tod](#tod)
  - [Table of Contents](#table-of-contents)
  - [Installation](#installation)
    - [Crates.io (Linux, Mac, and Windows)](#cratesio-linux-mac-and-windows)
    - [AUR (Arch-based Linux)](#aur-arch-based-linux)
    - [GitHub (Linux, Mac, and Windows)](#github-linux-mac-and-windows)
  - [Usage](#usage)
    - [Discovering the commands](#discovering-the-commands)
    - [Usage Examples](#usage-examples)
    - [Shell script examples](#shell-script-examples)
      - [Sort, schedule, prioritize, and process tasks](#sort-schedule-prioritize-and-process-tasks)
  - [How task priority is determined](#how-task-priority-is-determined)
  - [Disabling spinners](#disabling-spinners)
  - [Why I made this](#why-i-made-this)
  - [Related projects](#related-projects)
  - [Contributing](#contributing)
<!--toc:end-->


Will ask for your [Todoist API token](https://todoist.com/prefs/integrations) on the first run, and your data in JSON format in `$XDG_CONFIG_HOME/tod.cfg`. This defaults to:

- `~/.config/tod.cfg` on Linux
- `~/Library/Application Support/tod.cfg` on Mac
- No idea about Windows, sorry!

## Installation

### Crates.io (Linux, Mac, and Windows)

[Install Rust](https://www.rust-lang.org/tools/install)

```bash
# Linux and MacOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install Tod

```bash
cargo install tod
```

### AUR (Arch-based Linux)

```bash
# Use yay or another AUR helper
yay tod-bin
```

### GitHub (Linux, Mac, and Windows)

[Install Rust](https://www.rust-lang.org/tools/install)

Clone the project

```bash
git clone git@github.com:alanvardy/tod.git
cd tod
./test.sh # run the tests
cargo build --release
```

You can then find the binary in `/target/release/`

## Usage

### Discovering the commands

```bash
> tod -h

A tiny unofficial Todoist client

Usage: tod [OPTIONS] [COMMAND]

Commands:
  task     
  project  
  version  
  help     Print this message or the help of the given subcommand(s)

Options:
  -o, --config <CONFIGURATION PATH>  Absolute path of configuration. Defaults to $XDG_CONFIG_HOME/tod.cfg
  -q, --quickadd <quickadd>...       Create a new task with natural language processing.
  -h, --help                         Print help
  -V, --version                      Print version
```

And also use it to dig into subcommands

```bash
> tod task -h

Usage: tod task <COMMAND>

Commands:
  create    Create a new task
  list      List all tasks in a project
  next      Get the next task by priority
  complete  Complete the last task fetched with the next command
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```


### Usage Examples

```bash
# Quickly create a task
tod -q Buy more milk today

# Create a new task (you will be prompted for content and project)
tod task create

# Create a task in a project
tod task create --content "Write more rust" --project code

# Import your projects
tod project import

# Get the next task for a project
tod task next

# Go through tasks with an interactive prompt, completing them in order of importance one at a time.
tod project process

# Complete the last "next task" and get another
tod task complete && tod task next

# Get your work schedule
tod tasks list --scheduled --project work

# Get all tasks for work
tod tasks list --project work
```

### Shell script examples

#### Sort, schedule, prioritize, and process tasks

```bash
  echo "" && \
  echo "=== EMPTYING INBOX ===" && \
  tod project empty --project inbox && \
  echo "" && \
  echo "=== SCHEDULING DIGITAL ===" && \
  tod project schedule --project digital && \
  echo "" && \
  echo "=== SCHEDULING PHYSICAL ===" && \
  tod project schedule --project physical && \
  echo "" && \
  echo "=== PRIORITIZING DIGITAL ===" && \
  tod project prioritize --project digital && \
  echo "" && \
  echo "=== PRIORITIZING PHYSICAL ===" && \
  tod project prioritize --project physical
  echo "" && \
  echo "=== PROCESSING DIGITAL ===" && \
  tod project process --project digital && \
  echo "" && \
  echo "=== PROCESSING PHYSICAL ===" && \
  tod project process --project physical;
```

### Update Tod only if it is out of date

```bash
tod version check || cargo install tod --force
```

## How task priority is determined

Tasks are ranked by points and the first is returned, the points are the sum of the following:

  - Task is overdue: 150
  - The date is today with no time: 100
  - The date is today with time in next or last 15 min: 200
  - No date: 80
  - Not recurring: 50
  - Task has no priority: 2
  - Priority 1: 1
  - Priority 2: 3
  - Priority 3: 4

## Disabling spinners

Find the line in your `tod.cfg` that reads `"spinners": null` and change the value to false.

## Why I made this

I am a developer who uses Todoist to reduce stress and cognitive overhead, by delegating things that a machine does well to a machine. This CLI application scratches some very specific itches for me, and I hope that it may be of use to others as well!

Some points around my general strategy:

- Do one thing at a time, multi-tasking is an illusion (see `tod project process`)
- Capture all tasks immediately with the inbox and add detail later (see `tod project empty`, `schedule`, and `prioritize`)
- Make all your tasks "actions", concrete tasks that can be acted on. Add phone numbers, hyperlinks etc to your tasks
- Batch process like things as infrequently as possible to lower context switching, i.e. clear your email inbox once per day, spam once per week.
- Remember that the objective is to **get the important things done with less friction**, not just get more things done.
- Further to the above point, make sure to leave yourself margin. It is in the spaces between the periouds work that we recover and get our best ideas.
- Less projects are better than more projects
- Use projects as "modes" where you only work in one at a time

## Related projects

- [Alfred Workflow](https://github.com/stacksjb/AlfredTodWorkflow)

## Contributing

Contributions are welcome, just please open up an issue before putting too much work into a PR.
