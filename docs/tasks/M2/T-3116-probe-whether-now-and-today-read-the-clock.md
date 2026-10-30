---
id: T-3116
title: "Probe whether NOW and TODAY read the clock once per recalc or once per call, and settle SPEC-0008's non-goal"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence on whether Excel takes one clock instant per recalculation, so that every `NOW()` in a recalc is the
same serial, or reads the clock at each call. SPEC-0008 then states the rule as Excel behavior instead of
"unprobed". If the engine's `Clock::System` behaves differently, the follow-up engine task exists.

## Context
- docs/excel-semantics/dates.md "NOW() keeps sub-second precision" ends: "Unprobed: whether Excel reads its clock
  once per recalc, so that two `=NOW()` cells and `=NOW()-NOW()` agree exactly, or once per call. RustSheet's system
  clock reads once per call".
- docs/specs/SPEC-0008-workbook-clock.md lists "One instant per recalc" as a non-goal for the same reason. Today
  `Model::now_unix_millis` (crates/rustsheet-engine/src/model.rs) reads `Clock::now_unix_millis` at each NOW/TODAY
  call (crates/rustsheet-engine/src/clock.rs, `Kind::System` → `SystemTime::now()`). The harness pins
  `Clock::fixed_unix_millis(HARNESS_CLOCK_MS)` (crates/rustsheet-eval/src/engine.rs), so no eval cell can show the
  difference today. The question matters for embedders on the system clock. TODAY/NOW are M2 scope
  (docs/milestones.md).
- Why opus: the evidence depends on timing. A per-call Excel can still give `NOW()=NOW()` TRUE when both calls
  fall within one clock tick, so a single TRUE proves nothing. The probe must put slow work between two reads.
  Examples: a long `SUM(SEQUENCE(..))` or `MAKEARRAY` cell between two NOW cells that are compared in a third cell;
  `LET` with a slow binding between two NOW bindings; two NOW cells on different sheets. Choose sizes that take
  well over Excel's clock resolution but stay inside the oracle's limits. The run must be repeated (at least three
  oracle fills) before a TRUE counts as "once per recalc".
  Only cells that are the same on every run may be scored. List them in the case's `invariant` field
  (crates/rustsheet-eval/src/case.rs, T-1283). Do not score a raw `NOW()-NOW()` difference that varies.
- Method: the `excel-probe` skill (excel-graph). Note the backend, the dates and the run count in the evidence line.

## Acceptance criteria
- [x] A probe case `probes/t3116-now-once-per-recalc` holds the comparison cells above, built so that a per-call
      clock gives FALSE in at least one of them on a typical run. It is oracle-filled at least three times, the
      results are recorded, and its scored cells are listed in `invariant` and are the same on every fill.
- [x] dates.md replaces the "Unprobed" sentence with the rule and its evidence (VERIFIED, or UNVERIFIED with the
      reason if the runs disagree). SPEC-0008's "One instant per recalc" bullet is updated to match.
- [x] If Excel reads once per recalc, an engine follow-up task (M2, engine) is filed. It makes `Clock::System` give
      one instant per recalc and names this case's cells. If Excel reads per call, the engine needs no change and the
      task says so in its Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine clock (the follow-up task).
- RAND/RANDBETWEEN/RANDARRAY, which are SPEC-0008 non-goals, and the timezone model.
- Desktop Excel (M14, decision 0039).

## Outcome
- probes/t3116-now-once-per-recalc was filled four times with excel-graph on 2026-10-30. The first
  repeat fills returned the cached workbook unchanged, so three fills changed an inert cell (Z1)
  to make Excel recalculate afresh. On every fill, two NOW reads with about 50 ms of dependent work
  between them differ by 20 to 50 ms, in 10 ms steps: D1, A2, A4 and A7 are FALSE. NOW()-NOW()=0
  (A5) and INT(TODAY())=INT(NOW()) (A6) are TRUE.
- Per fill, the gaps E1 / A3 / Other!A2 were 30/50/30 ms (the committed sidecar), then
  20/40/20, 30/30/30 and 30/30/30 ms (Z1 = "run 2", "run 3", "run 4").
- Rule: Excel reads its clock per call, not once per recalc. dates.md and SPEC-0008 now say so
  (VERIFIED). The engine's `Clock::System` already reads per call, so no engine follow-up is needed.
- Only A5 and A6 are scored (`invariant`). The per-call cells are recorded, not scored: the
  harness's fixed clock gives one instant per recalc, and those cells cannot show the difference.
- Eval delta: +2 scored cells, both passing; 0 new known failures; 0 regressions.
