---
id: T-0443
title: "eval check reports the probes baseline out of date on main: 2 new-coverage failures (1632 -> 1633)"
milestone: M11
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-eval check` on main reports no stale probes baseline, or the cause of the drift is found and the baseline updated with `make eval-baseline` by the change that caused it.

## Context
Seen 2026-10-20 while working T-0442 (after `rustsheet-eval fetch --suite poi`): `rustsheet-eval check` prints `[probes] known failures 1632 -> 1633, text 2 -> 2 (fixed 0, new-coverage failures 2, REGRESSIONS 0)` and `baseline is out of date`, with the same output when T-0442's change is stashed. Probe milestone M0-M2 also reads 12459 cells and 1 known failure against 12474 and 0 at origin/main earlier the same day. Find which merge moved it (`git log eval/baseline/probes.txt`; recent: T-0982, T-2181, T-0969) and whether it is a real new failing cell (a task for it) or a baseline that was not refreshed.

## Acceptance criteria
- [ ] The cause is named in the Outcome, with a task filed if a cell newly fails.
- [ ] `rustsheet-eval check` shows no `baseline is out of date` line for probes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
No change needed: the drift is gone on current `origin/main`. With poi fetched, `rustsheet-eval check` reports `[probes] known failures 1630 -> 1630, text 2 -> 2 (fixed 0, new-coverage failures 0, REGRESSIONS 0)`, no `baseline is out of date` line, and visible parity 1744 = lock. The merge that left the baseline stale was not identified (the stale reading was of a main since superseded, and the baseline-only history does not show which change moved two cells); the later baseline-refreshing merges to `eval/baseline/probes.txt` (T-0982 #1361, T-2174 #1355) brought it back in line. No cell newly fails, so no task is filed. T-0443-style drift can recur when a merge does not run `make eval-baseline`; `make check` in CI is the gate for it.
