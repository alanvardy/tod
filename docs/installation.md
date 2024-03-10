# Installation

<!--toc:start-->
- [Installation](#installation)
  - [Crates.io (Linux, Mac, and Windows)](#cratesio-linux-mac-and-windows)
  - [AUR (Arch-based Linux)](#aur-arch-based-linux)
  - [GitHub (Linux, Mac, and Windows)](#github-linux-mac-and-windows)
<!--toc:end-->

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

## AUR (Arch-based Linux)

```bash
# Use yay or another AUR helper
yay tod-bin
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

