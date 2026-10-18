---
id: T-0056
title: "Evidence floor: report and ratchet in-scope functions with fewer than 20 oracle cells, and file probe batches for them"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
No in-scope function has thin Excel evidence. Every in-scope catalog function has at least 20
oracle-backed cells, and the report and a ratchet keep it that way.

## Context
Owner request (2026-10-15): finish Phase 1's test suite before the Phase 2 re-architecture
(decision 0027). Building test infrastructure is allowed during Phase 1, and this is one of the
suites the milestone exit criteria name that does not exist yet.
- On 2026-10-14, `make eval` on main showed 83 in-scope functions with fewer than 20 checked
  cells. That count includes cached-value suites, and the median was 59.
- The report already has per-function evidence (`report.json` `evidence`, T-0020/T-0051).

## Acceptance criteria
- [x] The report lists in-scope functions below the floor (20 oracle or Excel-cached cells),
  grouped by category.
- [x] `quality/` gains a ratchet on that count, changed only through its update tooling, so it
  can only go down. This touches a protected path, so it needs a decision record.
- [x] The listed functions are split into probe tasks of ≤ 10 functions each (research lane,
  sonnet tier where no numeric fitting is involved). Each asks for cases covering every
  parameter mode and the catalog's edge pools.
- [x] `make check` green; reviewers approve.

## Out of scope
- Writing the probes themselves (the filed batch tasks do that).

## Outcome
`rustsheet-eval evidence-floor` (`make evidence-floor`, in `make check` and CI's eval job) counts
in-scope, oracle-evaluable catalog functions with fewer than 20 Excel cells over the vendored
suites and ratchets the count in `quality/evidence-floor.json` (decision 0062); `make
eval-baseline` records drops. The report's evidence section gains "Below the evidence floor",
grouped by catalog category (the catalog view now carries `category`); the old 25-cell
threshold became the single 20-cell `EVIDENCE_FLOOR`. Count on main: 73. Probe batches filed:
T-1620..T-1628 (research, sonnet, ≤ 10 functions each). Eval delta: none (harness only).
