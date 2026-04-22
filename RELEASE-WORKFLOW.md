# Release Workflow

This repository publishes releases through a GitHub Actions workflow in `.github/workflows/release.yml`.

## How releases work

- The workflow is triggered only by pushing a Git tag that starts with `v`, for example `v0.1.5`.
- When a tag is pushed, GitHub Actions builds release binaries for Linux, Windows, and macOS.
- After a successful build, the workflow publishes the crate to crates.io using `cargo publish`.
- The workflow also creates a GitHub release and uploads the generated binaries.

## What you must do before tagging

1. Update the version in `Cargo.toml`.
   - The crate version is read from the `version = "..."` field.
   - This version is not automatically incremented by the workflow.
2. Commit the updated `Cargo.toml`.
3. Create and push a tag matching the new version, such as `v0.1.5`.

Example:

```bash
git add Cargo.toml
git commit -m "Bump version to 0.1.5"
git tag v0.1.5
git push origin main --tags
```

## Important notes

- The workflow publishes whatever version is currently set in `Cargo.toml`.
- If the pushed tag value and `Cargo.toml` version disagree, the repository still publishes the `Cargo.toml` version.
- The release workflow does not itself bump `Cargo.toml`.

## Workflow location

- `.github/workflows/release.yml`
- `.github/workflows/prepare-release.yml`

## Automatic release preparation

A new workflow is available at `.github/workflows/prepare-release.yml`.
Use it to bump `Cargo.toml` and create the release tag automatically.

### How to use

1. Open the repository Actions tab in GitHub.
2. Select the `Prepare Release` workflow.
3. Run it with the desired `version` input, for example `0.1.5`.
4. Optionally provide a custom `tag`; otherwise it defaults to `v<version>`.

The workflow will update `Cargo.toml`, commit the change, create the tag, and push both to the remote.

## Summary

Release = bump version + push `v*` tag → GitHub Actions builds + publishes.
Use `prepare-release` to automate the version bump and tag creation.
