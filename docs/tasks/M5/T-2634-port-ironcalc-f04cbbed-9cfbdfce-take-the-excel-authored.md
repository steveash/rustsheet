---
id: T-2634
title: "Port ironcalc f04cbbed/9cfbdfce: take the Excel-authored ROUND_ROUNDUP_ROUNDDOWN.xlsx calc test workbook"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The ironcalc-calc suite scores upstream's Excel-authored `ROUND_ROUNDUP_ROUNDDOWN.xlsx`, and every
newly scored ROUND/ROUNDUP/ROUNDDOWN cell matches Excel or is a recorded known failure with an owner.

## Context
Upstream IronCalc f04cbbed ("extend xslx tests ... for ROUND, ROUNDUP, ROUNDDOWN", 2026-10-21)
replaces `xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/ROUND.xlsx` (13,069 bytes) with
`ROUND_ROUNDUP_ROUNDDOWN.xlsx`, and 9cfbdfce (2026-10-23) re-saves it (17,276 bytes). docProps/app.xml
reads `Microsoft Excel` 16.0300, so its cached values are oracle evidence once you confirm the new
cells are Excel-computed (calcChain, 17-digit caches), not written by IronCalc. Our copy is
`crates/rustsheet-xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/ROUND.xlsx`. 9cfbdfce also adds an
Excel lock file `~$ROUND.xlsx` (165 bytes): do not take it. Triaged by the 2026-10-25 steward run.

## Acceptance criteria
- [ ] Our `ROUND.xlsx` is replaced by upstream's 9cfbdfce `ROUND_ROUNDUP_ROUNDDOWN.xlsx` (not the
  `~$ROUND.xlsx` lock file); `make eval` reports the new cells; any excel-semantics citation of the
  old file is re-pointed.
- [ ] New failures are fixed in `functions/math_and_trigonometry/` or recorded with
  `make eval-baseline` and filed as follow-up tasks; no regressions.
- [ ] Commit carries `Ported-From: ironcalc@9cfbdfce`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Upstream's `test_fn_round.rs` unit tests are IronCalc-authored, not Excel evidence; take only
  those whose expectations an Excel cell confirms.

## Outcome (provisional, pending the decision record below)
Replaced `ROUND.xlsx` with upstream's Excel-authored `ROUND_ROUNDUP_ROUNDDOWN.xlsx` (9cfbdfce;
the `~$ROUND.xlsx` lock file not taken). Checked cells 137 -> 326, all pass; no known failures
added, 0 regressions. Re-pointed the numbers.md citation. IronCalc-authored unit tests not taken.

## Handoff (resumed)
Work is complete and reviewed (security, architecture, quality, excel-fidelity all APPROVE; make check
green locally; ironcalc-calc case 137 -> 326 cells, all pass, 0 regressions). CI `hygiene` fails:
`scripts/check_governance.py` flags `ROUND.xlsx` as a deleted test file ("requires a decision record and
reviewer sign-off", a protected path, so beyond a sonnet run). Next: `make new-decision TITLE="..."`
recording that upstream renamed ROUND.xlsx to ROUND_ROUNDUP_ROUNDDOWN.xlsx (superset, every old cell
retained) so the delete is a rename, commit it, push, re-run reviewers on the decision, merge the PR
(#1899) under the merge lock, then set status: done.
