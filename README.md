## Tod

[![Build Status](https://github.com/alanvardy/tod/workflows/check/badge.svg)](https://github.com/alanvardy/tod)
[![Build Status](https://github.com/alanvardy/tod/workflows/test/badge.svg)](https://github.com/alanvardy/tod)
[![Build Status](https://github.com/alanvardy/tod/workflows/rustfmt/badge.svg)](https://github.com/alanvardy/tod)
[![Build Status](https://github.com/alanvardy/tod/workflows/clippy/badge.svg)](https://github.com/alanvardy/tod)

A tiny todoist CLI program. Takes simple input and dumps it in your inbox or another project. Tasks sent to the inbox can take advantage of natural language processing to assign due dates, tags etc.

Will ask for your Todoist API token on first run, and store the token in json format in `~/.tod.cfg`. You can obtain your token from [Todoist Preferences](https://todoist.com/prefs/integrations).

Also stored with your token is a mapping of project names to project ids, i.e.

```json
{"projects":{"project_name":12345678},"token":"a09999999999dd999fe8a48c07fd3c99999999ac07"}
```

### Install

Clone the project

```bash
git clone git@github.com:alanvardy/tod.git
```

asdf install rust and build the release

```bash
cd tod
asdf install
cargo build --release
```

You can then find the binary in `/target/release/`

Add an alias! I use ZSH and store my github projects in `~/coding/` and thus added this line to my `~/.zshrc`:

```bash
alias tod="~/coding/tod/target/release/tod"
```

### Run

```bash
# you can use inbox, in or i to send items to your inbox
# tasks sent to the inbox can use natural language processing
tod inbox Buy milk from the grocery store tomorrow

# send it to a project defined in ~/.tod.cfg
# tasks sent to projects dont use natural language processing, because API.
tod project_name write more rust
```
