---
name: gh-context
description: Analyze GitHub issues and PRs, pick the best actionable work, and return only high-confidence triage recommendations.
---

# gh-context

Use this skill to triage GitHub issue or pull request context and produce a short ranked list of the best work to pick next.

This skill is not a general summary tool. It should only recommend issues that are actionable, unowned, and supported by a clear fix direction.

## When to use

- Selecting the best issue or PR from a backlog.
- Turning issue context into a decision-ready recommendation.
- Finding work that can be picked up without lengthy investigation.
- Exporting issue context as markdown or JSON for downstream workflows.

## Requirements

- `gh` must be installed and authenticated.
- `gh-context` should be available in `PATH`.
- If it is not installed globally, run it from the repo with `cargo run -- <input> [OPTIONS]`.

## Use the tool

1. Choose whether the input is a single issue, a single PR, a PR range, or a bulk issue export.
2. Run `gh-context <input>` when the binary is installed.
3. Use `--issue` or `--pr` if shorthand input is ambiguous.
4. Use `--format json` for programmatic consumption; otherwise use markdown.
5. Use `--out <path>` to save output, or `--clip` on macOS to copy it.
6. Use `--bulk` with `--state`, `--pages`, and `--per-page` for repository-wide exports.
7. In triage mode, do more than restate the issue: explain the failure, the likely subsystem, and a likely fix direction.
8. Do not recommend work that appears owned, blocked, or too uncertain.

## Triage criteria

- Prefer issues with a clear repro, logs, linked code, or a narrow failure scope.
- Prefer issues that suggest a concrete engineering next step.
- Prefer issues with enough evidence to outline a likely fix.
- Deprioritize speculative problems, missing repro details, or broad design requests.
- Deprioritize stale issues unless there is recent confirmation they still matter.
- Exclude issues with an open PR that appears to be the fix.
- Exclude issues with explicit ownership signals: assigned people, "working on this", claimed, draft PRs, or linked fix branches.
- Exclude otherwise-owned issues unless the user specifically requests assigned work.

## What to analyze

For each candidate issue, explain:

- The core problem in one or two sentences.
- The likely affected subsystem or root cause.
- Why this issue is a strong candidate now.
- A plausible fix path.
- The main unknowns, blockers, or risks.

Only recommend an issue if you can say something concrete about both the problem and the fix direction.

## Confidence rating

Each recommended issue must include `confidence_fix: 1-5`:

- `1`: Very low confidence; the issue is unclear and likely needs major investigation.
- `2`: Low confidence; some direction exists but the fix is still uncertain.
- `3`: Moderate confidence; a reasonable path exists but notable unknowns remain.
- `4`: High confidence; the issue is scoped and a credible fix path is visible.
- `5`: Very high confidence; repro, subsystem, and implementation direction are clear.

Only issues rated `4` or `5` should appear in the recommended shortlist. Lower-confidence candidates belong in `Needs more investigation` or should be omitted.

## Recommended output format

When triaging multiple issues, return a ranked shortlist with:

- Issue reference and title
- Why it is worth picking now
- Problem analysis
- Potential solution direction
- `confidence_fix: <4|5>`
- Ownership status: `unowned`, `possibly owned`, or `owned`
- Blockers or missing information

If an issue is owned, put it under `Do not pick now` with the ownership signal.

If an issue is promising but below `4`, put it under `Needs more investigation`.

## Extra checks

- Look at labels for difficulty, priority, regression, good first issue, or help wanted.
- Review recent comments for maintainer guidance or activity.
- Check whether the issue is blocked on another issue or PR.
- Check whether the fix requires cross-team design or product input.
- Prefer candidates with clearer validation criteria and a narrower blast radius.

## Output guidance

- Prefer markdown for human-readable agent conversations.
- Prefer JSON when another tool will parse the result.
- Keep the shortlist short and focused; decision quality is better than exhaustiveness.
- Only recommend issues with `confidence_fix: 4` or `5`.
