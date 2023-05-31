# Publish Checklist

## Setup `tod-bin`

Create `tod-bin` directory for pushing to AUR

```bash
./setup_aur.sh
```

## Publish to Cargo and AUR

This checklist is just here for me to reduce the friction of publishing new versions.

Code changes

1. Change version in `Cargo.toml`
2. Update dependencies and make sure nothing broke with `./update_test.sh`
3. Update CHANGELOG.md with version number
4. Update README.md with help text `cargo run -- -h`
5. Add any new examples to README.md
6. Open PR for version and wait for it to pass
7. Commit and merge PR

8. Build release

```bash
git checkout main
git pull
cargo aur
```

9. [Create a new release](https://github.com/alanvardy/tod/releases/new)

- Make sure to use the label and title in format `v0.3.8`
- Add binary from `tod` directory

10. Publish to Cargo with `cargo publish`
11. Push to AUR with `./push_aur.sh`

