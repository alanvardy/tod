# Publish Checklist

This checklist is just here for me to reduce the friction of publishing new versions.

- Change version in Cargo.toml
- Update CHANGELOG.md with version number
- Update README.md with help text `cargo run -- -h`
- Add any new examples to README.md
- Build cargo release with `cargo aur`
- [Create a new release](https://github.com/alanvardy/tod/releases/new) and add binary
- Create .SRCINFO with `makepkg --printsrcinfo > .SRCINFO`
