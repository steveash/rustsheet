---
id: T-2296
title: "Probe SUMPRODUCT: does an error element in a range beat an earlier shape mismatch"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
Oracle evidence for which error SUMPRODUCT returns when an argument holds an error *element* (inside a range or
array, not a whole-argument error) and the argument shapes disagree.

## Context
T-2150 established (probes/t2200-blank-and-padded-m5-args A20-A23) that a whole-argument error or blank one-cell
argument beats an earlier shape mismatch. For an error element ours keeps the old order: the first mismatched argument
stops element reading, so `SUMPRODUCT(D1:D2,<2x2 with #DIV/0!>)` is `#VALUE!`, while an error element in an argument
ahead of the mismatch wins. Neither is probed (docs/excel-semantics/coercion.md "SUMPRODUCT: a blank one-cell
argument", Open line; code `fn_sumproduct` in crates/rustsheet-engine/src/functions/math_and_trigonometry/sumproduct.rs,
`TODO(T-2296)`). Use the `excel-probe` skill; suggested cells, with a 2x2 range R holding one `#DIV/0!` element and
a 2x1 range S of numbers: `SUMPRODUCT(S,R)`, `SUMPRODUCT(R,S)`, `SUMPRODUCT(S,R,NA())`, `SUMPRODUCT(R,S,NA())`,
`SUMPRODUCT(S,{1,#N/A;2,3})`, `SUMPRODUCT({1,#N/A;2,3},S)`, `SUMPRODUCT(S,R2,R)` with R2 a 2x2 of numbers.

Evidence from gen-m7 (T-2254 triage, excel-graph, 2026-10-22): `BYROW({1,#N/A,3},LAMBDA(acc,SUMPRODUCT(Data!B1:C4,acc)))` is `#N/A`
(f7e94a817e5385ac A50): SUMPRODUCT of a 4x2 range and the 1x3 row {1,#N/A,3} (one BYROW slice) gives the error element of the *later*,
mismatched argument, not `#VALUE!`. Ours `#VALUE!`. So an error element beats a shape mismatch even when it sits in the argument that
mismatches; the probe should still separate the orders listed above. Owned by the functions-math fix task this probe files: 1 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas.
- gen-m7/f7e94a817e5385ac.case.json A50 `=BYROW({1,#N/A,3},LAMBDA(acc,SUMPRODUCT(Data!B1:C4,acc)))`: A50 Excel #N/A, ours #VALUE!; cells (1): A50

## Acceptance criteria
- [ ] A probe case in eval/suites/probes with oracle values for the cells above.
- [ ] Visible parity: the gen-m7 cell under Context (f7e94a817e5385ac A50) is owned by the implementation task this probe files (listed there, `campaign: visible`); its known-failure line leaves eval/baseline/gen-m7.txt when that task is done.
- [ ] coercion.md entry's Open line replaced by the probed rule; an implementation task filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing any change (file a functions-math task if needed).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
