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

Releases
- [Create a new release](https://github.com/alanvardy/tod/releases/new) and add binary

```bash
cargo publish
makepkg --printsrcinfo > ../tod-bin/.SRCINFO
mv PKGBUILD ../tod-bin/
cd ../tod-bin/
git add .
git commit -m 0.2.6
git push aur
```
