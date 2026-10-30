---
id: T-3217
title: "SPEC-0020 amendment: a lifted VLOOKUP, HLOOKUP or MATCH whose lookup_value is a whole column evaluates its tail once (probes/t3096-lookup-value-blank-elements takes 4.8 s)"
milestone: M11
lane: perf
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
SPEC-0020 has an accepted amendment that says how a lookup lifted over a whole-column `lookup_value` (`VLOOKUP(A:A,..)`,
`HLOOKUP(A:A,..)`, `MATCH(A:A,..)`, alone or under INDEX) does work proportional to the used rows plus one row for
the tail, with bit-identical results. The amendment also files its Rollout tasks.

## Context
Gap triage 2026-10-30 (`make eval` on origin/main): probes/t3096-lookup-value-blank-elements passes, but it is the
slowest case outside the lunar-calendar probes (T-2908): 4.8 s for 203 cells. Measured on a scratch split of the case
(`rustsheet-eval --root <scratch> show`, release build):
- its 32 formulas with a whole-column `lookup_value` (Calc!B44:E51, for example
  `=INDEX(VLOOKUP(Keys!$A:$A,Data!$A$1:$B$5,2,FALSE),1000)` and `=INDEX(MATCH(Keys!$A:$A,Data!$A$1:$A$5,1),2)`) take
  5.1 s, about 160 ms each;
- the other 171 formulas take 20 ms.

Each such call lifts the lookup over 1,048,576 elements, nearly all blank past the used area, and builds a
1,048,576-row result that INDEX then picks one element from. At this rate a sheet with a few hundred such formulas
(`INDEX(MATCH(A:A,..),n)`, `SUM(--(VLOOKUP(A:A,..)=x))`) takes about a minute.

SPEC-0020 covers this shape only in part:
- its non-goals exclude "the criteria functions and lookups", because they "already clip to the used area or index
  (T-0448, T-0449)". That clipping applies to the *searched* range, not to a lifted `lookup_value`;
- Amendment 1 (T-0488) added LOOKUP's vector form, UNIQUE, SMALL and LARGE as consumers, but not VLOOKUP, HLOOKUP,
  MATCH or XLOOKUP's lifted key.

Design questions the amendment must answer:
- Which lifted calls produce a tailed result. The key's blank tail gives one repeated result row: the lookup of a blank
  key, which t3096 pins against tables holding 0, empty text, FALSE and blanks, exact and approximate.
- Which consumers read a tailed result without materializing it: INDEX with a constant row, the aggregate walk
  (already a SPEC-0020 consumer), and a spill. A spill still materializes, and its size checks are unchanged.
- How the lift cost and the work budget (SPEC-0024, SPEC-0028) are charged: the logical size, as SPEC-0020 charges
  them today.
- Seen-empty records for the key's tail (`clip_to_used_area`): unchanged, one rectangle.

Code: `functions/lift/`, `spill_functions.rs`, `tailed_array.rs`; the lookups in `functions/lookup_and_reference/`.

## Acceptance criteria
- [ ] docs/specs/SPEC-0020-whole-column-array-operands.md has an "Amendment 2" section with the Problem (the timings
      above), the design answers above, the invariants per consumer, and a test plan that names
      probes/t3096-lookup-value-blank-elements and its expected run time after the Rollout (under 0.5 s).
- [ ] Its Rollout tasks are filed with `make new-task` (lane `perf`, one PR each) and linked from the amendment.
- [ ] No engine change in this task; `make check` green; reviewers approve the amendment.

## Out of scope
- Implementing the amendment (its Rollout tasks).
- XLOOKUP and XMATCH, unless the amendment finds they share the same lift path. If they do, they go in as one more
  Rollout task.
