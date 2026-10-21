---
id: T-2245
title: "Visible-parity lock: PRs stop committing drops, the steward lowers it daily (open PRs all conflicted on it)"
milestone: M5
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Parallel fix PRs stop conflicting on `eval/baseline/visible-parity.json`, so finished PRs reach CI
and merge instead of sitting idle with no event to wake their authors.

## Context
- Decision 0088 required the count to equal the lock, so every fix PR committed a new number into
  the same one-line file. On 2026-10-21, 7 of the 21 idle open PRs conflicted on it, and 6 on
  nothing else (#1403, #1393, #1386, #1381, #1378, #1353). GitHub runs no CI on a conflicting PR,
  so no event reaches the author.
- crates/rustsheet-eval/src/visible_parity.rs `judge`, main.rs `visible_parity_gate`.

## Acceptance criteria
- [x] `check` passes a count at or below the lock and fails a rise.
- [x] `update-baseline` writes the lock only for a rise that new coverage accounts for (or with
      `--allow-regressions`), and records a drop only with `--lower-visible-lock`.
- [x] `make eval-lower-visible-lock` exists; the steward skill runs it daily; docs updated
      (quality-gates.md, eval-harness.md); decision 0093.
- [x] `make check` green; reviewers approve.

## Out of scope
- Per-cell visibly-off markers in the baselines (the owner chose daily lowering instead).

## Outcome
`judge` takes an `Update { allow_regressions, lower }`; the `Stale` judgement is gone, and `Holds`
covers both equal and below-lock counts. Decision 0093 amends 0088. The open PRs that conflicted
only on the lock resolve by taking `main`'s side.
