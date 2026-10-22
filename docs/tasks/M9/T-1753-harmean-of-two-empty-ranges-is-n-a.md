---
id: T-1753
title: "HARMEAN of a range with no numbers is #N/A, not #DIV/0!"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
HARMEAN of arguments with no numbers (one range of blank cells) is #N/A, not #DIV/0!.

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q50 (`HARMEAN(D3:D4)`): Excel #N/A, ours #DIV/0!.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 1 cell, 0 numeric past half a unit in the 15th digit and 1 with a wrong type, error or text, blamed on HARMEAN 1. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the visibly-off cell listed under Context (2026-10-21) has Excel's value to 15 significant digits, or its type, error and text: the 1 line in eval/baseline/probes.txt disappears (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#DIV/0!` where Excel is `#N/A`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Outcome
`fn_harmean` returns `#N/A` when no numbers remain (was `#DIV/0!`). probes known failures -1 (stats-evidence-floor-t1623 S!Q50), 0 regressions. Unit test `test_fn_harmean_no_numbers_is_na`; statistics.md Ours line updated.
