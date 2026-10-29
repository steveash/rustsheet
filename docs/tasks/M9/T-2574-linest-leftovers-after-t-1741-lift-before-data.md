---
id: T-2574
title: "LINEST leftovers after T-1741: lift before data errors (TRIMRANGE as reference), nested-call error order"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1741]
---

## Goal
The gen-m9 LINEST cells that T-1741 left failing match Excel.

## Context
T-1741 (docs/excel-semantics/statistics.md "LINEST and LOGEST: shapes, one point, collinear columns, exact fits and
check order") left these owned cells failing (excel-graph, 2026-10-22/23):
- Lift order. `LINEST(<#DIV/0!>,Data!C2:E7,..,MAKEARRAY(2,3,..))` is `#SPILL!` (gen-m9 c2974c061384fdd9 A89), and so is
  `LINEST(Data!C7:C8,<#VALUE!?>,FILTER(..),DROP(SEQUENCE(1,1),-3,-1))` (ee2497a5e683c5d9 A319). So an array const or stats
  lifts before the data arguments' error values, as TREND's does. Ours reports the data error first, because
  `LOGEST(Data!C6:E12,<#NUM!>,TRIMRANGE(Data!A7:A12),TRUE)` is `#NUM!` (0a614cfe974b59c3 A342). That is consistent
  if Excel's TRIMRANGE is a reference (a range does not lift) while ours returns an array: check TRIMRANGE's
  result kind first.
- Error choice in nested or mixed calls (ours first):
  - `LINEST(Data!A7:A8,(..+PROB(..)),2,INDEX(Data!A1:C6,5))` Excel `#N/A`, ours `#VALUE!` (4250f880ab640c49 A224).
  - `LINEST(Z.TEST(..))` `#VALUE!` vs `#DIV/0!` (8f2ac25945af87da A5).
  - `LINEST(DROP(..),Data!B10:C12,"FALSE",0.5)` `#NUM!` vs `#VALUE!` (a5828d3925575713 A320).
  - c7664e494e418395 A152 `#REF!` vs `#VALUE!`; 2138b909d8f810ea A142 `#CALC!` vs `#VALUE!`; 92d4fa29e5510f1e A222
    `#DIV/0!` vs `#N/A`; 139684e68fb2fc3b A11 and 9d9ceba257f8001b A346 `#NUM!` vs `#REF!`; 01523d247594952e A6
    `#NUM!` vs `#VALUE!` (an outer XIRR may be the cause).
- gen-m9/589f42e67f4b4b8d A192 `LINEST(Data!A10:B11)`: Excel 27.839, ours 27.875 (a 2-D y read element by element;
  check the default x order) and b6593d7dd13f2dac A395 (close).
Find which inner call diverges first (`make eval-show`), fix the LINEST rule where it is LINEST's, and hand the rest
to the inner function's task.

T-1741 review notes (2026-10-24, minor, for this task): the "typed text const is `#VALUE!` at once, a text cell waits"
rule is written twice (LINEST's `flag_input`/`flag_value` in linest.rs and TREND's inline `typed` in trend.rs): move it into
regression.rs when touching either. A multi-cell range as const or stats falls back to the default (as before T-1741):
probe `LINEST(y,x,A1:A2)` and `LINEST(y,x,TRUE,A1:B1)`. The collinear fallback (`independent_columns`, O(p·n·kept)) and the
range read have no work budget beyond the normal equations' O(n·p²); consider a cell budget (directive 5) when the kernel
is revisited.
- Cells this task owns that an earlier owner rule credited to it or to another task only by an A1
  name written beside a different case (decision 0159, T-0099):
  - gen-m9/b8ee55934c5b824c.case.json: Calc!A161
  - gen-m9/c2974c061384fdd9.case.json: Calc!A89

## Acceptance criteria
- [ ] The cells listed above pass or name another open task that owns their root cause; their lines leave
      eval/baseline/gen-m9.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The solve's last digits (T-2006). (gen-m9/c468f5046aadfbbc A122/B122 pass since T-2547.)
