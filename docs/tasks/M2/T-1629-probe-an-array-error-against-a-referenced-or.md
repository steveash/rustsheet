---
id: T-1629
title: "Probe an array error against a referenced or direct error in GEOMEAN, HARMEAN, SKEW, KURT, VAR, STDEV, AVERAGEA, MAXA and MINA"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for how an error inside an array ranks against a referenced or a direct error in GEOMEAN, HARMEAN, SKEW, KURT, VAR, STDEV, AVERAGEA, MAXA and MINA, with ours fixed if it differs.

## Context
T-1615 probed SUM, MIN, MAX, AVERAGE and MEDIAN (probes/aggregate-array-error-vs-ref-error: the leftmost of the referenced, array and LAMBDA errors wins, a direct error beats them all) and made the shared walker hold array errors for every `Ranked` policy, which also changed these functions: UNVERIFIED extrapolation (syntax.md "Aggregate error precedence"). Extend the probe's D1-D40 shapes to them; the A-variants have their own reading rules (`A_ARGS`).

## Acceptance criteria
- [ ] A probe with oracle values for the shapes of D1-D40 on each function; syntax.md updated; ours fixed if it differs
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `aggregate-array-error-vs-ref-error-average-family` (87 cells, excel-graph): the nine functions rank like SUM in all eight shapes; a direct LAMBDA is `#VALUE!` in AVERAGEA, MAXA, MINA (ours skipped it; fixed in `direct_aggregate_number`). syntax.md updated. Eval: probes +87 cells, all pass.

## Handoff (resumed)
- Done and reviewed (security, architecture, quality, excel-fidelity all APPROVE), `make check` green locally: probe `aggregate-array-error-vs-ref-error-average-family` (87 cells, excel-graph), the A-variant direct-LAMBDA fix in `direct_aggregate_number`, syntax.md, baseline.
- Failing: CI `hygiene` job, `scripts/check_governance.py origin/main`: "protected paths changed without a decision record in docs/decisions/: quality/evidence-floor.json". `make eval-baseline` lowered `below_floor` 63 -> 60 (tool-written, not hand-edited).
- Next: record the 63 -> 60 drop in a decision record (`make new-decision`; T-1627 logged its 73 -> 63 drop in decision 0010, follow that), have reviewer-architecture and reviewer-security approve that part explicitly, push, wait for CI, merge per the steward skill. PR #862. Task status goes back to `done` with the Outcome kept once merged-ready.
- Resumed (opus): logged the 63 → 60 evidence-floor drop in decision 0010, merged main.
