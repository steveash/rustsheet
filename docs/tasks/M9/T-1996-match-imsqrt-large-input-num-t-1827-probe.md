---
id: T-1996
title: "Match IMSQRT large-input #NUM! (T-1827 probe A33, A34)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
IMSQRT of a large real input gives what Excel gives: `#NUM!` for `IMSQRT("1E307")` and `IMSQRT("4E307")`.

## Context
Known failures probes/eng-im-overflow S!A33, S!A34 (oracle `#NUM!`; ours returns text). Find the threshold with a probe first (excel-probe skill), then implement in `imsqrt` (functions/engineering/complex.rs). See docs/excel-semantics/engineering.md "IM* results that overflow".

Visibly-off cells (gap triage 2026-10-21, decision 0083): 2 cells, 0 numeric past half a unit in the 15th digit and 2 with a wrong type, error or text, blamed on IMSQRT 2. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the 2 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 2 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A33 and A34 pass and the threshold is pinned by a probe; no finite IMSQRT result changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (IMSQRT prints "infNaNi" where Excel is `#NUM!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
