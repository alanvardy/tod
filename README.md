# Tod

[![Build Status](https://github.com/alanvardy/tod/workflows/ci/badge.svg)](https://github.com/alanvardy/tod) [![codecov](https://codecov.io/gh/alanvardy/tod/branch/main/graph/badge.svg?token=9FBJK1SU0K)](https://codecov.io/gh/alanvardy/tod) [![Crates.io](https://img.shields.io/crates/v/tod.svg)](https://crates.io/crates/tod)

A tiny todoist CLI program. Takes simple input and dumps it in your inbox or another project. Takes advantage of natural language processing to assign due dates, tags, etc.

![Tod](tod.gif)

Will ask for your [Todoist API token](https://todoist.com/prefs/integrations) on the first run, and your data in JSON format in `$XDG_CONFIG_HOME/tod.cfg`. This defaults to:

- `~/.config/tod.cfg` on Linux
- `~/Library/Application Support/tod.cfg` on Mac
- No idea about Windows, sorry!

## Install from Crates.io

[Install Rust](https://www.rust-lang.org/tools/install)

```bash
# Linux and MacOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install Tod

```bash
cargo install tod
```

## Install from AUR (for Arch-based Linux distributions)

```bash
# Use yay or another AUR helper
yay tod-bin
```

## Install from GitHub

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

Start with the help flag to get the latest commands

```bash
> tod -h

A tiny unofficial Todoist client

Usage: tod [OPTIONS] [COMMAND]

Commands:
  task     
  project  
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

- Add your most commonly used projects, the project ID is the last series of numbers in the URL. If the project name includes spaces, wrap the project name with quotes.
- You can use natural language processing such as dates priority etc when sending to inbox, but not to the projects due to current limitations.
- Items are ranked by points and the first is returned:
  - Item is overdue: 150
  - The date is today with no time: 100
  - The date is today with time in next or last 15 min: 200
  - No date: 80
  - Not recurring: 50
  - Item has no priority: 2
  - Priority 1: 1
  - Priority 2: 3
  - Priority 3: 4

### Examples

```bash
# Quickly create a task
tod -q Buy more milk today

# Create a new task (you will be prompted for content and project)
tod task create

# Create a task in a project
tod task create --content "Write more rust" --project code

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
