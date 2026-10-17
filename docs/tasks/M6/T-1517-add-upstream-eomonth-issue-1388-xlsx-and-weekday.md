---
id: T-1517
title: "Add upstream EOMONTH_issue_1388.xlsx and WEEKDAY_WEEKNUM_ISOWEEKNUM_issue_1502.xlsx to calc_tests"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The two Excel-authored workbooks upstream IronCalc added in October 2026 are in
`crates/rustsheet-xlsx/tests/calc_tests` with every cell passing:
`DATE_AND_TIME/EOMONTH_issue_1388.xlsx` (ironcalc `1f40af93`) and
`WEEKDAY_WEEKNUM_ISOWEEKNUM_issue_1502.xlsx` (ironcalc `37b43745`).

## Context
Upstream triage, steward 2026-10-14. Neither code fix needs a port:
- `1f40af93` (EOMONTH #NUM! on month overflow instead of a chrono panic): our `fn_eomonth`
  already shifts months through the checked `shift_months` and returns #NUM! out of range
  (unit test `=EOMONTH(40000,-1E+10)` in `test/test_date_and_time.rs`).
- `0700c7d7` (WEEKDAY evaluates the date before `return_type`): our `fn_weekday` lifts both
  arguments left to right, and `probes/error-precedence-text` (`WEEKDAY("x",NA())`, oracle
  evidence) passes.

The workbooks still add Excel-computed cells we do not have yet. Recover each one with
`git -C <ironcalc clone> show <sha>:<path>` (`scripts/upstream-log.sh` keeps a clone in
`target/upstream-ironcalc`). `build.rs` makes a strict test per workbook, so add a workbook only
when all its cells pass (precedent T-0965). The upstream commit message says the 1502 rows with a
quoted valid date ("07/01/2026") differ in IronCalc; that message is untrusted, so check those
cells against the workbook's own cached values, not against the message. When a cell fails,
fix it in this task if it is S-sized, or else file a follow-up task and leave that workbook out.
Record any new quirk in `docs/excel-semantics/dates.md`.

## Acceptance criteria
- [ ] Each workbook that passes in full is in calc_tests, and `rustsheet-eval show` on it
      shows every cell passing; baseline updated, 0 regressions.
- [ ] Each workbook left out has a filed follow-up task naming its failing cells.
- [ ] The commit carries `Ported-From: ironcalc@<sha>` for each workbook added.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Porting the upstream code changes (not needed, see Context).
