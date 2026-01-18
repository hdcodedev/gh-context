# gh-context

A CLI tool to fetch GitHub Issue and Pull Request context.

<p align="center">
  <img width="551" height="452" alt="Screenshot 2026-01-18 at 11 23 56" src="https://github.com/user-attachments/assets/b86419cb-49cd-4b3c-a1de-ba8a5a0341f5" />
</p>

## Prerequisites

This tool requires the GitHub CLI (`gh`) to be installed and authenticated.

```bash
# macOS
brew install gh

# Authenticate
gh auth login
```

## Installation

To install the tool globally (usually to `~/.cargo/bin`):

```bash
cargo install --path .
```

## Usage

### Running Installed Command

Once installed, you can use `gh-context` directly:

```bash
gh-context <input> [OPTIONS]
```

### Running Locally (Development)

You can run the tool without installing it using `cargo run`. Note the `--` separator used to pass arguments to the CLI.

```bash
cargo run -- <input> [OPTIONS]
```

### Examples

Fetch context for a PR (creates `repo-issue-123/repo-issue-123.md` context by default, where `repo` is the repository name):
```bash
gh-context owner/repo#123
```

Fetch context for an issue and copy to clipboard:
```bash
gh-context https://github.com/owner/repo/issues/123 --clip
```

Save as JSON (prints to stdout):
```bash
gh-context owner/repo#123 --format json
```

Using `cargo run`:
```bash
cargo run -- https://github.com/hdcodedev/resume256/issues/48
```

### Options

- `--format <json|md>`: Output format (default: md)
- `--out <path>`: Write output to file
- `--clip`: Copy output to clipboard (macOS only)
- `--issue`: Treat input as issue (disambiguate shorthand)
- `--pr`: Treat input as PR (disambiguate shorthand)
