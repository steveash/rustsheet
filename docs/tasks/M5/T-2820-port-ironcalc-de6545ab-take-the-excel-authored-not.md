---
id: T-2820
title: "Port ironcalc de6545ab: take the Excel-authored NOT.xlsx calc test workbook"
milestone: M5
lane: functions-logic
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The ironcalc-calc suite scores upstream's Excel-authored `NOT.xlsx`, and every newly scored NOT
cell matches Excel or is a recorded known failure with an owner.

## Context
Upstream IronCalc de6545ab ("add test cases for NOT", 2026-10-26) adds
`xlsx/tests/calc_tests/LOGICAL/NOT.xlsx` (16,745 bytes). Its docProps/app.xml reads
`Microsoft Macintosh Excel` 16.0300 and it has a calcChain, so its cached values are oracle
evidence once you confirm the cells are Excel-computed (17-digit caches), not written by IronCalc.
We have no NOT workbook yet; add it next to `crates/rustsheet-xlsx/tests/calc_tests/LOGICAL/AND_OR_XOR.xlsx`.
Upstream's companion fix 7ec21c74 (NOT lifts over arrays and ranges) is already in our
`fn_not` (logical/and_or_xor_not.rs, T-0225, T-0260), so expect most cells to pass. Precedent:
T-1517, T-2634. Triaged by the 2026-10-26 (18:47 UTC) steward run.

## Acceptance criteria
- [ ] `LOGICAL/NOT.xlsx` from de6545ab is in the ironcalc-calc suite; `make eval` reports its cells.
- [ ] New failures are fixed in `functions/logical/` or recorded with `make eval-baseline` and
  filed as follow-up tasks; no regressions.
- [ ] Commit carries `Ported-From: ironcalc@de6545ab`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Upstream's `test_fn_not_array.rs` unit tests are IronCalc-authored, not Excel evidence.

## Outcome
Took upstream's `LOGICAL/NOT.xlsx` (de6545ab, Microsoft Macintosh Excel cache, 16,745 bytes).
The ironcalc-calc suite now scores 115 checked cells for it, all pass; no known failures
added (the baseline gained only the new checked-cell count line), 0 regressions.
