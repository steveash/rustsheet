---
id: T-1395
title: "COUNTA counts the blank elements of an IFERROR array but not those of TAKE of a blank range"
milestone: M2
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
COUNTA counts the empty elements of an array made by IFERROR/IFNA over a range, while still skipping the blank of TAKE over a blank range. (A lifted INDEX over a range with a blank cell counts it: probes/index-lifted-edge-cases H136, T-1437.)

## Context
probes/counta-empty-value D18: `COUNTA(IFERROR(B1:B2,1))` (B1 blank, B2 "x") is 2 in Excel, the engine gives 1. Counting every `ArrayNode::Empty` fixes it but regresses probes/subtotal-reference-operands-c H13 (`COUNTA(TAKE(B3:B3,1))` of a blank is 0), so the two empties must be told apart in the array representation (an IFERROR empty value vs a blank cell). Design question, hence opus.

## Acceptance criteria
- [x] D18 passes with no regression of subtotal-reference-operands-c H13.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
COUNTA reads each argument in picked-reference context (`evaluate_picked_reference_arg`), so TAKE, DROP, INDEX and
OFFSET over a range stay references whose blank cells are skipped, and it counts every element of an array value,
the empty values IFERROR gives for blank cells included. No new array representation was needed: the two empties
were already told apart by whether the argument is a reference. Counting array empties broke the upstream
`BYCOL(A1:C3,LAMBDA(col,COUNTA(col)))` test, because BYCOL handed the LAMBDA arrays: a new excel-graph probe
(probes/lambda-helper-range-slices) shows that BYROW, BYCOL, MAP, SCAN and REDUCE all hand a range's slices over as
references, so BYROW and BYCOL now do (`by_slices_source`). A range keeps its full size (a whole column gives
1048576 slices, probe G1:G4) under the shared `check_range_size` bound. MAP/SCAN/REDUCE references and the eta-reduced
`BYCOL(..,COUNTA)` go to T-1442. Eval: probes known failures 2753 → 2759: fixed 2 (D18 and
probes/index-lifted-edge-cases H136, the COUNTA half of T-1437), and the new probe case adds 46 scored cells with
8 new-coverage failures (T-1442). 0 regressions in any suite (poi included).
