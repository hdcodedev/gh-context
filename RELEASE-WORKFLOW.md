# Release Workflow

This repository publishes releases through a GitHub Actions workflow in `.github/workflows/release.yml`.

## How releases work

- The workflow runs on push to `main` when `Cargo.toml` changes.
- It checks whether the `version = "..."` value in `Cargo.toml` changed in that push.
- If the version changed, GitHub Actions publishes the crate to crates.io using `cargo publish`.

## What you must do before publishing

1. Update the version in `Cargo.toml`.
   - The crate version is read from the `version = "..."` field.
2. Merge that version change to `main`.

Example:

```bash
git add Cargo.toml
git commit -m "Bump version to 0.1.5"
git push origin main
```

## Important notes

- The workflow publishes whatever version is currently set in `Cargo.toml` on `main`.
- The release workflow does not itself bump `Cargo.toml`; use `prepare-release` for that.

## Workflow location

- `.github/workflows/release.yml`
- `.github/workflows/prepare-release.yml`

## Automatic release preparation

A new workflow is available at `.github/workflows/prepare-release.yml`.
Use it to bump `Cargo.toml` and open a release PR automatically.

### How to use

1. Open the repository Actions tab in GitHub.
2. Select the `Create Release PR` workflow.
3. Run it with `version_bump` set to `patch`, `minor`, or `major`.
4. Merge the generated `release/v<version>` PR.

The workflow updates `Cargo.toml`, pushes a release branch, and opens a PR to `main`. Merging that PR triggers publish automatically.

## Summary

Release = bump version + merge release PR to `main` → GitHub Actions publishes.
Use `prepare-release` to automate the version bump and release PR creation.
