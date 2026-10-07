---
id: T-0269
title: "Probe LOOKUP edges: single-cell lookup_vector, scalar or function result_vector, reading past the sheet edge"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-0224]
---

## Goal
Oracle evidence for the LOOKUP vector-form edges T-0224 left as engine choices, so each becomes
VERIFIED or is fixed.

## Context
T-0224 (docs/excel-semantics/lookup.md "LOOKUP vector form with 2-D or short vectors") verified 2-D
and short vectors, but chose these without evidence:
- a single-cell lookup_vector (`LOOKUP(2,C1,D1)`, `LOOKUP(0,C1,D1)`): the engine says `#N/A`;
- a non-reference scalar result_vector (`LOOKUP(2,A1:A2,5)`): `#N/A`;
- a function that returns a reference as result_vector (`LOOKUP(99,A1:A2,INDEX(F1:F1,1))`,
  `OFFSET(F1,0,0)`): is it read past its end like a plain `F1`?
- reading past the sheet's last row or column (`LOOKUP(99,A1:A3,F1048575:F1048576)`,
  `LOOKUP(99,A1:B1,XFD1)`): the engine says `#N/A`.
Use the `excel-probe` skill (one workbook for all of these is fine, since each formula parses).
Code: `fn_lookup`, `lookup_result` in crates/rustsheet-engine/src/functions/lookup_and_reference/lookup.rs.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes/ covers each bullet, with oracle values.
- [ ] lookup.md records the results; each engine mismatch has a fix task (or is fixed if trivial).
- [ ] `make check` green; reviewers approve.

## Out of scope
- LOOKUP's array form.
