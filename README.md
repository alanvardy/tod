## Tod

[![Build Status](https://github.com/alanvardy/tod/workflows/ci/badge.svg)](https://github.com/alanvardy/tod) [![codecov](https://codecov.io/gh/alanvardy/tod/branch/master/graph/badge.svg?token=9FBJK1SU0K)](https://codecov.io/gh/alanvardy/tod) [![Crates.io](https://img.shields.io/crates/v/tod.svg)](https://crates.io/crates/tod)

A tiny todoist CLI program. Takes simple input and dumps it in your inbox or another project. Tasks sent to the inbox can take advantage of natural language processing to assign due dates, tags etc.

![Tod](tod.gif)

Will ask for your [Todoist API token](https://todoist.com/prefs/integrations) on first run, and your data in json format in `~/.tod.cfg`. 


### Install from Crates.io

[Install Rust](https://www.rust-lang.org/tools/install)
```
# Linux and MacOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install Tod

```bash
cargo install tod
```

### Install from GitHub

[Install Rust](https://www.rust-lang.org/tools/install)

Clone the project

```bash
git clone git@github.com:alanvardy/tod.git
cd tod
./test.sh # run the tests
cargo build --release
```

You can then find the binary in `/target/release/`

### Usage

Start with the help flag to get the latest commands

```
> tod -h

Tod 0.2.4
Alan Vardy <alan@alanvardy.com>
A tiny unofficial Todoist client

USAGE:
    tod [FLAGS] [OPTIONS]

FLAGS:
    -c, --complete      Complete the last task fetched with next
    -h, --help          Prints help information
    -l, --list          List all the projects in local config
    -n, --next          Get the next task by priority. Requires project option.
    -z, --prioritize    Assign priorities to tasks. Can specify project option, defaults to inbox.
    -e, --scheduled     Returns items that are today and have a time. Can specify project option, defaults to inbox.
    -s, --sort          Sort inbox by moving tasks into projects
    -V, --version       Prints version information

OPTIONS:
    -a, --add <PROJECT NAME> <PROJECT ID>    Add a project to config with id
    -t, --task <new task>...                 Create a new task with text. Can specify project option, defaults to inbox.
    -p, --project <PROJECT NAME>             The project namespace, for use with other commands
    -r, --remove <PROJECT NAME>              Remove a project from config by name
```

- You will be asked for an API key on first login, which is stored in `~/.tod.cfg`
- Add your most commonly used projects, the project ID is the last serials of numbers in the URL, the project name cannot include spaces.
- You can use natural language processing such as dates priority etc when sending to inbox, but not to the projects due to current limitations.
- Items are ranked by points and the first is returned:
  - Item is overdue: 150
  - Date is today with no time: 100
  - Date is today with time in next or last 15 min: 200
  - No date: 80
  - Not recurring: 50
  - Item has no priority: 2
  - Priority 1: 1
  - Priority 2: 3
  - Priority 3: 4

#### Examples

```bash
# Create a new task in inbox using natural language processing
tod -t Buy milk from the grocery store tomorrow p1

# Create a task in a project
tod -p myproject -t write more rust \\ with a description

# Get the next task for a project
tod -np myproject

# Complete the last "next task" and get another
tod -c && tod -np myproject

# Get your work schedule
tod -ep work
```
