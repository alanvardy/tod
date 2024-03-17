# Usage

<!--toc:start-->
- [Usage](#usage)
  - [Discovering the commands](#discovering-the-commands)
  - [Usage Examples](#usage-examples)
  - [Shell script examples](#shell-script-examples)
    - [Sort, schedule, prioritize, and process tasks](#sort-schedule-prioritize-and-process-tasks)
  - [Update Tod only if it is out of date](#update-tod-only-if-it-is-out-of-date)
  - [How task priority is determined](#how-task-priority-is-determined)
<!--toc:end-->

## Discovering the commands

```bash
> tod -h

A tiny unofficial Todoist client

Usage: tod [OPTIONS] [COMMAND]

Commands:
  task     
  project  
  filter
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


## Usage Examples

```bash
# Quickly create a task
tod -q Buy more milk today

# You can use Todoist syntax with the quickadd (q) command
# See https://todoist.com/help/articles/use-task-quick-add-in-todoist-va4Lhpzz for more details
tod -q Buy more milk today // with a description

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

## Shell script examples

### Sort, schedule, prioritize, and process tasks

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

## Update Tod only if it is out of date

```bash
tod version check || cargo install tod --force
```
