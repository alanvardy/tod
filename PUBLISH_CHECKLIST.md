# Publish Checklist

## Setup `tod-bin`

Create `tod-bin` directory for pushing to AUR

```fish
./setup_aur.sh
```

## Experimental Publish Procedure

1. Update `CHANGELOG.md` with version number
2. Create PR with

```fish
VERSION=0.6.19 ./create_pr.sh
```

3. Wait for it to pass, then merge and pull in latest changes

```fish
gh pr merge -r --admin && gs
```

4. Release it to all the places

```fish
VERSION=0.6.19 ./release.sh
```

## Publish to Cargo and AUR (older method)

This checklist is just here for me to reduce the friction of publishing new versions.

Code changes

1. Change version in this file
2. Change version in `Cargo.toml`
3. Update and test with `./update_test.sh`
4. Update `CHANGELOG.md` with version number
5. Add any new examples to documentation
6. Open PR for version and wait for it to pass
7. Commit and merge PR
8. Build release

```bash
git checkout main
git pull
cargo aur
```

9. Create a new release

```
set VERSION "v0.6.16"
gh release create "$VERSION" ./target/cargo-aur/*.tar.gz --title "$VERSION" --generate-notes
```

10. Publish to Cargo with `cargo publish`
11. Push to AUR with `./push_aur.sh`
12. Delete any merged branches with `git-delete-merged-branches --yes`

