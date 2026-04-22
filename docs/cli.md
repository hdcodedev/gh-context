# gh-context CLI Documentation

> **Note**: Direct CLI usage is not recommended for most users. Prefer installing the agent skill with `npx skills add hdcodedev/gh-context` and using the `/gh-context triage` slash command instead.
>
> This documentation is for advanced users who need the standalone binary.

A CLI tool to fetch and format GitHub Issues and Pull Requests context, ready for use in LLM prompts.

## Prerequisites

This tool requires the GitHub CLI (`gh`) to be installed and authenticated.

```bash
# macOS
brew install gh

# Authenticate
gh auth login
```

## Installation

### Install the agent skill via `npx`

For the cleanest install flow, add the `skills.sh` skill from this repository:

```bash
npx skills add hdcodedev/gh-context
```

Update the skill:

```bash
npx skills update gh-context
npx skills remove gh-context
```

### Install the Rust CLI

Install from [crates.io](https://crates.io/crates/gh-context):

```bash
cargo install gh-context
```

Install from source locally:

```bash
cargo install --path .
```

## Usage

### Running the installed command

Once installed, use `gh-context` directly:

```bash
gh-context <input> [OPTIONS]
```

### Running locally for development

Use `cargo run` with `--` to pass arguments to the CLI:

```bash
cargo run -- <input> [OPTIONS]
```

## Examples

Fetch context for an issue (markdown output to stdout):

```bash
gh-context owner/repo#123
```

Bulk fetch open issues for a repo (one file per issue, first page by default, requires `--out`):

```bash
gh-context https://github.com/openai/codex/issues --bulk --out ./codex-issues
```

Bulk fetch multiple pages:

```bash
gh-context openai/codex --bulk --pages 3 --per-page 50 --out ./codex-issues
```

Fetch an issue and copy output to clipboard:

```bash
gh-context https://github.com/owner/repo/issues/123 --clip
```

Save as JSON:

```bash
gh-context owner/repo#123 --format json
```

Write single output explicitly to a file:

```bash
gh-context owner/repo#123 --out ./repo-issue-123.md
```

Use default smart inference with a natural-language request:

```bash
gh-context "find me one actionable issue"
```

## Options

- `--format <json|md>`: Output format (default: `md`)
- `--out <path>`: Write output to file or directory. Required with `--bulk`
- `--clip`: Copy output to clipboard (macOS only)
- `--issue`: Treat input as issue (kept for backwards compatibility, no longer needed)
- `--bulk`: Fetch multiple issues for a repo (list mode)
- `--state <open|closed|all>`: Issue state filter for bulk mode (`open` by default)
- `--per-page <n>`: Items per page for bulk mode (default: `30`)
- `--pages <n>`: Number of pages to fetch in bulk mode (default: `1`)

## Output behavior

- Single issue mode prints to stdout by default.
- Use `--out` to write a single issue result to a file.
- Bulk mode requires `--out` and writes files to the specified directory.

## Agent skill

This repository also includes an installable agent skill in `.agents/skills/gh-context/`.

### Skill install example

```bash
npx skills add hdcodedev/gh-context
```

### Example agent prompt

Use slash command form for the skill.

```text
/gh-context owner/repo#456
```

```text
/gh-context 123
Explain the best next step and why it is important.
```

If you are in a repository, you can also run it without arguments:

```text
/gh-context
```

This will infer the current repo from the git `origin` remote and select the top open issue candidate.
