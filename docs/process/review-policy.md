# Review policy

No human reviews PRs. Instead every PR is reviewed by independent subagents with narrow
mandates and fresh context. The worker that wrote the code only gives them the task and the diff.

| Reviewer | When | Blocks on |
|---|---|---|
| `reviewer-security` | always | panics/overflow on untrusted input, unbounded recursion/allocation, `unsafe`, I/O from evaluation, risky deps, secrets, CI permission/secret exposure, unjustified gate weakening |
| `reviewer-architecture` | always | layering violations, code in the wrong module, contradicting `docs/architecture` or a spec, unplanned public API changes, mixed-purpose PRs |
| `reviewer-quality` | always | redundant definitions (an equivalent helper already exists), new lint suppressions without reason, missing/weak tests, unreadable code, bare TODOs |
| `reviewer-excel-fidelity` | engine, xlsx, eval changes | semantics without oracle evidence, eval regressions, unrecorded quirks, edge cases (blank vs "", coercion, error precedence, precision) |
| `reviewer-performance` | hot paths, storage, graph, bulk APIs | asymptotic regressions, per-cell allocations, missing benchmark evidence, unsound dirty propagation |

## Verdicts
- `APPROVE`: mergeable. MINOR findings may be deferred to follow-up tasks.
- `REQUEST_CHANGES`: fix, then re-run that reviewer.
- `BLOCK`: the approach is wrong. Rethink it; if the disagreement persists, mark the task blocked.

A PR merges only when **all applicable reviewers APPROVE** and CI is green. The consolidated
verdict table goes into the PR body.

## Keeping reviewers honest
- Each reviewer must cite `path:line` and a concrete fix. Findings without them are ignored.
- The steward run spot-audits merged PRs weekly: it samples 3 merged PRs, re-reviews them with a
  stronger prompt, every reviewer on Opus, and records each missed issue in `docs/status.md` with
  the reviewer that missed it and the model it ran on. Repeated misses lead to reviewer prompt
  updates (via a decision record); a miss traced to a Sonnet reviewer reverts that reviewer to
  Opus (decision 0015).
