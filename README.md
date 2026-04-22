# gh-context

[![Crates.io](https://img.shields.io/crates/v/gh-context.svg)](https://crates.io/crates/gh-context)

An agent skill to triage GitHub Issues, providing high-confidence recommendations for actionable work.

## Prerequisites

1. Install GitHub CLI and authenticate:
```bash
brew install gh
gh auth login
```

2. Install the gh-context binary:
```bash
cargo install gh-context
```

## Installation

Install the agent skill with `npx`:

```bash
npx skills add hdcodedev/gh-context
```

Update it:

```bash
npx skills update gh-context
```

## Quick start

Just run this inside any GitHub repository and it will find you the best actionable issue:

```text
/gh-context
```

Automatically detects the repository from git origin and picks the top open issue candidate.

Example output:

```
### Recommended issue: #123 Fix crash on startup

**Why pick this now**: High impact crash affecting 20% of users, clear repro steps provided.

**Problem**: Application crashes immediately on launch when offline. Root cause is unhandled network error in initialization.

**Fix direction**: Add error handling around the network call in `src/init.ts:42`, fall back to cached config.

`confidence_fix: 5` 🟩 Obvious
`status: unowned`
```

## Examples

Use slash command form for the skill. `<input>` can be:
- An issue number in the current repo: `123`
- A full issue URL: `https://github.com/hdcodedev/gh-context/issues/123`
- Owner/repo#number shorthand: `hdcodedev/gh-context#123`

```text
/gh-context <input>
```

```text
/gh-context hdcodedev/gh-context#123
```

```text
/gh-context 123
Explain the best next step and why it is important.
```

## Quick notes

- Direct CLI usage is not recommended for most users. See [`docs/cli.md`](docs/cli.md) only if you need the standalone binary.
