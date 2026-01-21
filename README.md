# gh-context

[![Crates.io](https://img.shields.io/crates/v/gh-context.svg)](https://crates.io/crates/gh-context)

A CLI tool to fetch and format GitHub Issues and Pull Requests context, ready for use in LLM prompts.
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

To install from [crates.io](https://crates.io/crates/gh-context):

```bash
cargo install gh-context
```

To install from source (locally):

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

Bulk fetch open issues for a repo (one file per issue, first page by default):
```bash
gh-context https://github.com/openai/codex/issues --bulk
```

Bulk fetch multiple pages:
```bash
gh-context openai/codex --bulk --pages 3 --per-page 50
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
- `--out <path>`: Write output to file (single) or directory (bulk)
- `--clip`: Copy output to clipboard (macOS only)
- `--issue`: Treat input as issue (disambiguate shorthand)
- `--pr`: Treat input as PR (disambiguate shorthand)
- `--bulk`: Fetch multiple issues for a repo (list mode)
- `--state <open|closed|all>`: Issue state filter for bulk mode (default: open)
- `--per-page <n>`: Items per page for bulk mode (default: 30)
- `--pages <n>`: Number of pages to fetch in bulk mode (default: 1)
