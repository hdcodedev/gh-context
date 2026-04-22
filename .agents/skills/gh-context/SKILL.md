---
name: gh-context
description: Fetch and format GitHub issue or pull request context for agent workflows using the gh-context CLI.
---

# gh-context

Use this skill when you need to pull structured context from a GitHub issue or pull request and turn it into markdown or JSON for an agent prompt.

When using this skill for issue triage, the goal is not just to summarize issues. The goal is to identify the most actionable issues, explain why they matter, estimate whether they are realistically fixable, and avoid recommending items that are already actively owned.

## When to Use

- You need the full context for a GitHub issue or PR before implementation or review.
- You want a prompt-ready markdown summary instead of manually copying GitHub pages.
- You need to bulk export issue context from a repository.
- You need JSON output for downstream tooling.
- You need to triage a backlog and decide which issues are best suited for an agent or engineer to take next.

## Requirements

- GitHub CLI (`gh`) must be installed and authenticated.
- `gh-context` must be available in `PATH`.
- If the binary is not installed globally, run it from the repository with `cargo run --`.

## Steps

1. Determine whether the target is a single issue, a single PR, a PR range, or a bulk issue export.
2. Prefer `gh-context <input>` when the binary is installed.
3. If the binary is not available but this repository is present locally, use `cargo run -- <input> [OPTIONS]` from the repo root.
4. Use `--issue` or `--pr` if shorthand input is ambiguous.
5. Use `--format json` when the result needs to be consumed programmatically; otherwise prefer markdown.
6. Use `--out <path>` to persist output, or `--clip` on macOS when the result should go to the clipboard.
7. For repository-wide issue collection, use `--bulk` with `--state`, `--pages`, and `--per-page` as needed.
8. For triage work, do not stop at restating the issue. Analyze the failure mode, likely affected area, and a plausible fix direction.
9. Before recommending an issue, check for signals that someone is already working on it.
10. Rank only the issues that are both actionable and unowned.
11. Recommend only issues with `confidence_fix` of `4` or `5`.

## Triage Rules

- Prefer issues with a clear repro, logs, linked code area, or a narrowly scoped failure.
- Prefer issues that suggest a concrete next engineering step instead of vague product discussion.
- Prefer issues with enough evidence to propose an implementation direction.
- Deprioritize issues that are speculative, missing reproduction details, or likely to require broad design work before coding can start.
- Deprioritize stale issues when the current state is unclear unless there is recent confirmation that the problem still exists.
- Exclude issues that already have an open PR linked as the likely fix.
- Exclude issues where a maintainer or contributor clearly states they are actively working on it.
- Exclude issues assigned to someone else unless the user explicitly wants assigned work included.
- Treat "claimed", "working on this", "I can take this", linked draft PRs, linked branches, and recent implementation updates as ownership signals.

## What to Analyze

For each candidate issue, analyze:

- The core problem in one or two sentences.
- The likely root cause or subsystem involved.
- Why the issue is actionable now.
- A plausible fix approach, even if it is only a likely direction rather than a complete implementation plan.
- The main unknowns or blockers that could make the fix harder than it first appears.

Do not present an issue as a good candidate unless you can say something concrete about both the problem and the likely solution path.

## Confidence Rating

Every recommended issue must include a `confidence_fix` rating from `1` to `5`:

- `1`: Very low confidence. The issue is poorly specified or likely needs major investigation before coding.
- `2`: Low confidence. Some direction exists, but the likely fix is still uncertain.
- `3`: Moderate confidence. The affected area and a reasonable fix path are visible, but there are notable unknowns.
- `4`: High confidence. The problem looks well-scoped and the likely fix path is credible.
- `5`: Very high confidence. The repro, affected area, and implementation direction are all clear.

The rating should reflect confidence in landing a fix, not just confidence that the bug exists.

Only issues rated `4` or `5` should appear in the recommended shortlist. Issues rated `1`, `2`, or `3` can be mentioned only as non-picks, deferred candidates, or items needing more investigation.

## Recommended Output Format

When triaging multiple issues, produce a ranked shortlist. For each issue include:

- Issue reference and title.
- Why it is worth picking now.
- Problem analysis.
- Potential solution direction.
- `confidence_fix: <1-5>`
- Ownership status: `unowned`, `possibly owned`, or `owned`
- A short note on blockers or missing information.

If an issue appears owned, list it separately under `Do not pick now` with the ownership signal that caused the exclusion.

If an issue is promising but has `confidence_fix` below `4`, list it under `Needs more investigation` instead of recommending it.

## Extra Triage Checks

- Check labels for difficulty, priority, regression, good first issue, or help wanted.
- Check recent comments for maintainer guidance, changed requirements, or work already in progress.
- Check whether the issue is blocked on another issue or PR.
- Check whether the issue likely needs cross-team design or product input before implementation.
- Check whether the issue has enough testability that a fix can be validated.

If two issues are otherwise similar, prefer the one with clearer validation criteria and a narrower blast radius.

## Examples

```bash
gh-context owner/repo#123
gh-context https://github.com/owner/repo/issues/123 --clip
gh-context owner/repo#123 --format json
gh-context owner/repo --from 244 --to 276
gh-context openai/codex --bulk --pages 3 --per-page 50
```

## Output Guidance

- Prefer markdown when the context will be pasted into an agent conversation.
- Prefer JSON when another tool will parse the result.
- If writing files, choose a clear destination path so generated context is easy to find later.
- For triage, optimize for decision quality rather than exhaustiveness. A short ranked list with strong reasoning is better than a long weak list.
- The recommended shortlist should contain only `confidence_fix: 4` and `confidence_fix: 5` issues.
