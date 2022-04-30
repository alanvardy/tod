# Publish Checklist

## Setup tod-bin

Create tod-bin directory for pushing to AUR

```bash
cd ~/dev
git clone ssh://aur@aur.archlinux.org/tod-bin.git
cd tod-bin
git remote add aur ssh://aur@aur.archlinux.org/tod-bin.git
```

## Publish to Cargo and AUR

This checklist is just here for me to reduce the friction of publishing new versions.

Code changes

1. Run `cargo update` to make sure dependencies are up to date
2. Run `./test.sh` to make sure that didn't break anything
3. Change version in Cargo.toml
4. Change version in this document
5. Update CHANGELOG.md with version number
6. Update README.md with help text `cargo run -- -h`
7. Add any new examples to README.md
8. Open PR for version and wait for it to pass
9. Commit and merge PR

10. Build release

```bash
git checkout main
git pull
cargo aur
```

11. [Create a new release](https://github.com/alanvardy/tod/releases/new)

- Make sure to use the label and title in format `v0.2.12`
- Add binary from tod directory

12. Publish to Cargo

```bash
cargo build --release
cargo publish
```

13. Push to AUR

```
cargo aur
makepkg --printsrcinfo > ../tod-bin/.SRCINFO
mv PKGBUILD ../tod-bin/
rm *.tar.gz
cd ../tod-bin/
git add .
git commit -m v0.2.11
git push aur
```
