---
id: T-1755
title: "STANDARDIZE reads a direct TRUE as 1"
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
STANDARDIZE reads a direct TRUE as 1 (`STANDARDIZE(TRUE,3,2)` is -1).

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q58: Excel -1, ours #VALUE!. Text "5" already matches.
An array element reads the same way: probes/stat-later-argument-lift L14 `ROUND(INDEX(STANDARDIZE({1.5;TRUE},1,2),2),12)`
is 0 in Excel, ours `#VALUE!` (T-1290; the kernel is `standardize_of`, statistical/standardize.rs).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 2 cells, 0 numeric past half a unit in the 15th digit and 2 with a wrong type, error or text, blamed on STANDARDIZE 2. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the 2 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 2 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#VALUE!` where Excel reads TRUE as 1), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Outcome
STANDARDIZE reads a direct TRUE x as 1 (`standardize_of` uses `cast_number_ref` for x). mean and standard_dev are unprobed and keep the boolean-is-#VALUE! read. Eval: probes known failures -2 (stats-evidence-floor-t1623 Q58, stat-later-argument-lift L14), no regressions.
