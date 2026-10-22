---
id: T-2410
title: "ROMAN lifts over an array form (gen-m7 TOROW(ROMAN(561,<2x4>),2) is five DLXI, ours #N/IMPL!)"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
ROMAN lifts over an array argument, one call per element (the form, and by the same SPEC-0011 rule the number), as Excel does,
instead of returning `#N/IMPL!`.

## Context
Gap triage of the T-2254 gen-m7 sweep (excel-graph, 2026-10-22); 5 cells, 1 formula.
`TOROW(ROMAN(561,HSTACK(Data!B7:B7,{1,2;3,4},Data!A$6)),2)` is five "DLXI" (f7e94a817e5385ac A1563:E1563). The form is the 2x4 array
{123456789.123, 1, 2, 0.5; #N/A, 3, 4, #N/A} (HSTACK pads its shorter arguments with `#N/A`); the elements with forms 1, 2, 0.5, 3 and 4
are "DLXI" (561 has the same spelling in every form), the 123456789.123 element is `#VALUE!` (form above 4, numbers.md "ROMAN truncates
before its sign check") and the two pads stay `#N/A`, so TOROW's ignore 2 (errors) leaves the five strings. So ROMAN lifts over an array
form with per-element errors (arrays.md "Scalar arguments lift: shapes, element errors, and the Analysis ToolPak range rule"). Ours:
`ROMAN(561,{1,2})` is `#N/IMPL!` because ROMAN has no entry in the lifting table (eval/catalog/lift-kinds.json has none). Give both
parameters the `Lift` kind (`Function::arg_kinds`, functions/lift/kinds/), reusing the scalar kernel in
functions/math_and_trigonometry/arabic_roman.rs. A multi-cell range (worksheet-function rule: it lifts too) and an array number follow the
same table entry; only the array form is evidenced here, say so in the quirk entry. COMBINA's array result in gen-m7 is not like this
(T-2414 probes it), so leave COMBINA out.

Owned gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): 5 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas.
- gen-m7/f7e94a817e5385ac.case.json A1563 `=TOROW(ROMAN(561,HSTACK(Data!B7:B7,{1,2;3,4},Data!A$6)),2)`: A1563 Excel "DLXI", ours #N/IMPL!; cells (5): A1563:E1563

## Acceptance criteria
- [ ] Visible parity: these 5 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline.
- [ ] Unit tests: `ROMAN(561,{0,1,4,5})` spills "DLXI", "DLXI", "DLXI", `#VALUE!`; `ROMAN({1;4},0)` spills "I", "IV".
- [ ] numbers.md's ROMAN entry states the lift and names this task; eval/catalog/lift-kinds.json regenerated (`rustsheet-eval lift-kinds`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- COMBINA (T-2414), ARABIC.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
