---
id: T-0144
title: "Read formula number literals at 15 significant digits (probes/literal-digits)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Formula number literals are rounded to 15 significant digits when parsed, as Excel does (docs/excel-semantics/numbers.md "Formula number literals are read at 15 significant digits"), so probes/literal-digits C1-C6 pass.

## Context
Found by T-0141. probes/literal-digits C1-C6 are known failures: `=1.00000000000000049-1` gives 4.44E-16 (Excel 0); C3-C6 may also need Excel's subtraction cancellation snap (`0.1+0.2-0.3` = 0, check whether that is a separate existing task before implementing). C2 `MOD(2,1.77635683940025E-12)` gives 1.10845E-12, Excel 1.10890E-12 even with a 15-digit literal: find out whether the parser or MOD's argument handling is responsible.

## Acceptance criteria
- [x] probes/literal-digits cells that are literal-parsing only pass and leave eval/baseline/probes.txt; any remaining cell is split to its own task.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
No code change was needed. The 15-digit literal cut ("15-digit literal parsing", numbers.md) had
already landed on main, and C1-C3, C5 and C6 pass. The one remaining known failure, C4
`=0.1+0.2-0.3` (Excel 0, engine 5.55E-17), is near-zero snapping of the final subtraction, not
literal parsing. It is handed to T-0110, confirmed as the owner of that snap (its Context and acceptance criteria
now list C4). T-0110's draft PR #227 waits on the unmerged task PR #228 (outer parentheses).
Eval: no change.
