---
id: T-0082
title: "Unowned visibly-off cell report and a dirty-tree flag in the eval report run record"
milestone: M0
lane: harness
status: in-progress
priority: P3
size: S
tier: opus
depends_on: [T-0080]
---

## Goal
Two small gaps from the T-0080 review:
- (a) The steward can see which visibly-off cells are attributed to no function and have neither
  an owning task nor a directive-5 `DEVIATION` (decision 0118 item 6). Phase 1 cannot exit while
  such a cell exists, and today nothing lists them.
- (b) report.json `run` says whether the tree had uncommitted changes when the eval ran, so a
  page or history row built from a dirty tree is visibly not the named commit's result.

## Context
- Decision 0118 item 6: cells calling two or more distinct non-glue functions, and glue-only
  formulas with two or more distinct calls, are attributed to nobody (`sole_call` returns `None`,
  crates/rustsheet-eval/src/attribution.rs and closeness.rs). Decision 0114 item 1 still needs a
  filed task or a `DEVIATION` for each of them.
- Owners: `make exit-owners` already maps failing exit cells to owning tasks; reuse its matching
  rather than writing a second one. A `DEVIATION` is an accepted-deviations line (decision 0036).
- `run` is `RunMeta` in crates/rustsheet-eval/src/run_meta.rs (T-0080): `commit`, `date`,
  `skipped_suites`. `dirty` comes from `git status --porcelain` at the repository root (ignored
  files do not count), `null` when git is unavailable, like `commit`.
- The progress page (SPEC-0022) can show both later; this task only produces them.

## Acceptance criteria
- [ ] report.json (or a `rustsheet-eval` subcommand the steward runs) lists the visibly-off,
      unattributed Excel cells with no owning task and no `DEVIATION`, as `suite/case!cell` with
      the formula, plus their count; report.md prints the count on one line. It gates nothing.
- [ ] Unit tests: a two-specialised-call cell with no owner is listed; the same cell with an
      owning task or a `DEVIATION` is not; an attributed cell is never listed.
- [ ] report.json `run.dirty` is `true` with uncommitted tracked or untracked changes, `false` on
      a clean tree, `null` without git; a unit test covers the parsing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rendering either on docs/progress.md (a later progress-report task may).
- Filing tasks for the listed cells: the steward does that from the list.
