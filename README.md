## Tod

[![Build Status](https://github.com/alanvardy/tod/workflows/ci/badge.svg)](https://github.com/alanvardy/tod)

A tiny todoist CLI program. Takes simple input and dumps it in your inbox or another project. Tasks sent to the inbox can take advantage of natural language processing to assign due dates, tags etc.

Will ask for your [Todoist API token](https://todoist.com/prefs/integrations) on first run, and your data in json format in `~/.tod.cfg`. 


### Install from Crates.io

[Install Rust](https://www.rust-lang.org/tools/install)

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
cargo test
cargo build --release
```

You can then find the binary in `/target/release/`

### Usage

#### Add a project

```bash
tod --add myproject 12345678
tod -a myproject 12345678
```

#### Remove a project

```bash
tod --remove myproject
tod -r myproject
```

#### List projects

```bash
tod --list
tod -l
```

#### Create a new task

```bash
# you can use inbox, in or i to send items to your inbox
# tasks sent to the inbox can use natural language processing
tod inbox Buy milk from the grocery store tomorrow

# send it to a project defined in ~/.tod.cfg
# tasks sent to projects dont use natural language processing, because API.
tod myproject write more rust
```
