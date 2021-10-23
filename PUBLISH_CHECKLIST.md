# Setup checklist

Create tod-bin directory for pushing to AUR
```
mkdir ../tod-bin
cd ../tod-bin
git init
git remote add aur ssh://aur@aur.archlinux.org/tod-bin.git
git fetch aur
```

# Publish Checklist

This checklist is just here for me to reduce the friction of publishing new versions.

Code changes

- Change version in Cargo.toml
- Update CHANGELOG.md with version number
- Update README.md with help text `cargo run -- -h`
- Add any new examples to README.md
- Open PR for version and wait for it to pass
- Commit and merge PR

- Build cargo release with `cargo aur`
- [Create a new release](https://github.com/alanvardy/tod/releases/new)
  - Make sure to use the label and title in format v0.2.7
  - Add binary

```bash
rm *.tar.gz
mv PKGBUILD ../tod-bin/
cargo build --release
cargo publish
makepkg --printsrcinfo > ../tod-bin/.SRCINFO
cd ../tod-bin/
git add .
git commit -m 0.2.7
git push aur
```
