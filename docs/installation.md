# Installation

<!--toc:start-->
- [Installation](#installation)
  - [Crates.io (Linux, Mac, and Windows)](#cratesio-linux-mac-and-windows)
  - [GitHub (Linux, Mac, and Windows)](#github-linux-mac-and-windows)
<!--toc:end-->

## Homebrew (Linux, Mac, and Windows)

```bash
brew tap alanvardy/tod
brew install tod
```

## Crates.io (Linux, Mac, and Windows)

[Install Rust](https://www.rust-lang.org/tools/install)

```bash
# Linux and MacOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install Tod

```bash
cargo install tod
```

## GitHub (Linux, Mac, and Windows)

[Install Rust](https://www.rust-lang.org/tools/install)

Clone the project

```bash
git clone git@github.com:alanvardy/tod.git
cd tod
./test.sh # run the tests
cargo build --release
```

You can then find the binary in `/target/release/`

Will ask for your [Todoist API token](https://todoist.com/prefs/integrations) on the first run
