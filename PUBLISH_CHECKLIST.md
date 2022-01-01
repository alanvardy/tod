# Setup checklist

Create tod-bin directory for pushing to AUR
```
cd ~/dev
git clone ssh://aur@aur.archlinux.org/tod-bin.git
```

# Publish Checklist

This checklist is just here for me to reduce the friction of publishing new versions.

Code changes

1. Change version in Cargo.toml
2. Change version in this document
3. Update CHANGELOG.md with version number
4. Update README.md with help text `cargo run -- -h`
5. Add any new examples to README.md
6. Open PR for version and wait for it to pass
7. Commit and merge PR

8. Build release

```bash
git checkout master
git pull
cargo aur
```

9. [Create a new release](https://github.com/alanvardy/tod/releases/new)
  - Make sure to use the label and title in format v0.2.10
  - Add binary from tod directory

10. Publish to Cargo and push to AUR repository
```bash
cargo build --release
cargo publish
makepkg --printsrcinfo > ../tod-bin/.SRCINFO
mv PKGBUILD ../tod-bin/
rm *.tar.gz
cd ../tod-bin/
git add .
git commit -m v0.2.10
git push aur
```
