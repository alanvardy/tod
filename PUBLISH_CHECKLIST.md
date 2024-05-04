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
2. Update and test with `./update_test.sh`
3. Update `CHANGELOG.md` with version number
4. Add any new examples to documentation
5. Open PR for version and wait for it to pass
6. Commit and merge PR
7. Build release

```bash
git checkout main
git pull
cargo aur
```

8. [Create a new release](https://github.com/alanvardy/tod/releases/new)

- Make sure to use the label and title in format `v0.3.8`
- Add binary from `tod` directory

9. Publish to Cargo with `cargo publish`
10. Push to AUR with `./push_aur.sh`

