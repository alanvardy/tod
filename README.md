## Tod

A tiny todoist CLI program. Takes simple input and dumps it in your inbox, does not currently support natural language input.

Will ask for your Todoist API token on first run, and store the token in plaintext in `~/todoist_token.cfg`. You can obtain your token from [Todoist Preferences](https://todoist.com/prefs/integrations)

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
tod Buy milk from the grocery store
```
