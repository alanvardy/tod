# Publish Checklist

Publication of release is automatically handled by [release-please](https://github.com/googleapis/release-please) and Github Automations.

## Automatic release & publish Procedure

1. Open a Pull Request with the "autorelease: pending" tag (note: release-please will automatically do this for any new feat: commits)
2. Review the PR and ensure all appropriate checklist items have completed (CI Tests, Documentation, etc)
3. Merge the PR

Upon merging of a release-please PR, release-please will automatically create a Github release with the appropriate version tags.

4. After release, Github CI will automatically begin build of the release binaries, and the following will run:

- release_linux.yml (Builds Linux binaries and uploads to release assets)
- release_macos.yml (Builds Macos binaries and uploads to release assets)
- release_windows.yml (Builds Windows binaries and uploads to assets)

5. After release builds complete, the following will automatically run

- release_cargo.yml (Publishes the latest build to crates.io)
- release_homebrew.yml (Sends an update event to the homebrew-tod repository which triggers an update of the Forumla.yml file with the latest version)
- release_windows.yml (The scoop update step will run and open a PR to update the /bucket/tod.json file)

Ensure you manually merge/close the scoop PR to update the JSON file.

## If there are any failures

Failing steps can be manually and indivdiually re-run if needed by executing them from under the "Actions" Github tab.
