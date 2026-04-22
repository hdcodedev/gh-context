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

## Agent Skill

This repository now includes an installable agent skill for `skills.sh` in `.agents/skills/gh-context/`.

Install from GitHub:

```bash
npx skills add <owner>/gh-context
```

Check that the skill is visible before installing:

```bash
npx skills add <owner>/gh-context --list
```

The repository will show up on the `skills.sh` leaderboard once users install it through the `skills` CLI.

### Skill Examples (Simple)

1. Install the skill

```bash
npx skills add <owner>/gh-context
```

2. Ask your agent (plain language)

```text
Use gh-context to triage https://github.com/owner/repo/issues/123.
Return: pick_now, confidence_fix, ownership_status, likely_fix_direction.
```

3. Ask your agent (backlog)

```text
Use gh-context to triage open issues in owner/repo.
Recommend only confidence_fix 4-5 issues.
Exclude issues already owned or in progress.
```

4. Update or remove

```bash
npx skills update gh-context
npx skills remove gh-context
```

### Slash Command Examples (If Supported)

Not every agent supports slash commands. If yours does, use:

```text
/gh-context triage https://github.com/owner/repo/issues/123
```

```text
/gh-context triage owner/repo#456 --issue
```

```text
/gh-context triage owner/repo#456
Return fields: pick_now, confidence_fix, ownership_status, likely_fix_direction, blockers
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

Fetch context for a PR (prints markdown to stdout by default):
```bash
gh-context owner/repo#123
```

Fetch a PR range (inclusive, one file per PR, requires `--out`):
```bash
gh-context owner/repo --from 244 --to 276 --out ./repo-prs
```
If any PR in the range fails to fetch, the command continues and prints a failure summary.

Bulk fetch open issues for a repo (one file per issue, first page by default, requires `--out`):
```bash
gh-context https://github.com/openai/codex/issues --bulk --out ./codex-issues
```

Bulk fetch multiple pages:
```bash
gh-context openai/codex --bulk --pages 3 --per-page 50 --out ./codex-issues
```

Fetch context for an issue and copy to clipboard:
```bash
gh-context https://github.com/owner/repo/issues/123 --clip
```

Save as JSON (prints to stdout):
```bash
gh-context owner/repo#123 --format json
```

Write single output explicitly to a file:
```bash
gh-context owner/repo#123 --out ./repo-pr-123.md
```

Using `cargo run`:
```bash
cargo run -- https://github.com/hdcodedev/resume256/issues/48
```

### Options

- `--format <json|md>`: Output format (default: md)
- `--out <path>`: Write output to file (single) or directory (bulk/range). Required with `--bulk` and `--from/--to`
- `--clip`: Copy output to clipboard (macOS only)
- `--issue`: Treat input as issue (disambiguate shorthand)
- `--pr`: Treat input as PR (disambiguate shorthand)
- `--bulk`: Fetch multiple issues for a repo (list mode)
- `--state <open|closed|all>`: Issue state filter for bulk mode (default: open)
- `--per-page <n>`: Items per page for bulk mode (default: 30)
- `--pages <n>`: Number of pages to fetch in bulk mode (default: 1)
- `--from <n>`: Start PR number for range mode (inclusive, requires `--to`)
- `--to <n>`: End PR number for range mode (inclusive, requires `--from`)

### Output Behavior

- Single issue/PR mode prints to stdout by default (both markdown and JSON).
- Single issue/PR mode writes to a file only when `--out` is provided.
- Bulk mode and PR range mode require `--out` and never write to the current directory implicitly.
