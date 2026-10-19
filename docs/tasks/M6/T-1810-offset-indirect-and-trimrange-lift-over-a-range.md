---
id: T-1810
title: "OFFSET, INDIRECT and TRIMRANGE lift over a range or an array into arrays of references"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1241]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
OFFSET's rows/cols/height/width, INDIRECT's ref_text and a1, and TRIMRANGE's trim_rows/trim_cols lift over a
multi-cell range and an array as Excel does, each element the scalar call's result; CHOOSE's index lift moves onto
the SPEC-0011 driver (`index_num` `Lift`, the value arms `Whole`).

## Context
Split from T-1241, which lifted the value-returning lookup scalars (ADDRESS, HYPERLINK, LOOKUP, XMATCH, FORMULATEXT)
and the CHOOSECOLS/CHOOSEROWS index lists. What is left returns references per element, which `ArrayNode` cannot
hold (SPEC-0011 non-goal 2, open question 5). Write a short design note first (in this task or docs/specs/): an
element that is a one-cell reference becomes that cell's value; a multi-cell element is `#VALUE!`-or-`#SPILL!`
per the evidence (`INDEX(TRIMRANGE(A1:B5,{0;1}),1,1)` is `#SPILL!`, a nested array). Check references.md and
how INDEX's own index lift (T-1260, lookup.md "INDEX lifts over array and range indices") treats its elements as
values before choosing. Build on `lift_values` and `read_lift_arg` (functions/lift/), not on a new broadcast.

Failing cells (probes/lookup-argument-modes, excel-graph, 2026-10-09; `rustsheet-eval show
probes/lookup-argument-modes.case.json`): F14:H15 INDIRECT, F20:H21 OFFSET, F34:H35 TRIMRANGE.
Generated cells (`rustsheet-eval show gen-m7/<case>.case.json`, gen-m9 likewise):
- gen-m7/cb748b6a9b81fbb7 Calc!A28 `=OFFSET(..,OFFSET(Data!D6:D9,3,(Data!D$4+7)))` (width a 4x1 reference, cols
  `#N/A`): Excel `#SPILL!` (the lift wins over the `#N/A` scalar argument), engine `#N/A`.
- gen-m9/675763979c1defb6 Calc!A25 `=UNICODE(INDIRECT("Data!A2:A4",VSTACK(Data!C6:E7)))` (a1 over an array):
  Excel `#SPILL!`, engine `#N/IMPL!`.
- gen-m9/adb3fd63c7368166 Calc!A34 `=INDIRECT(OFFSET(Data!A3:A5,3,1,3,3),(TANH(1)=Data!B12))` (ref_text over a
  3x3 range): Excel `#SPILL!`, engine `#VALUE!`.
Unprobed (lookup.md): OFFSET's cols, height and width, INDIRECT's a1, TRIMRANGE's trim_cols; probe them
(excel-probe skill) before entering them. XMATCH's search_mode (still a plain scalar, `#N/IMPL!` over an array) is
unprobed too: probe and lift it here.

## Acceptance criteria
Moved on the split (Outcome below), so left unchecked here: the first two items to T-1956 (OFFSET) and T-1957
(INDIRECT, TRIMRANGE), the third to T-1959.
- [ ] probes/lookup-argument-modes F14:H15, F20:H21 and F34:H35 pass; baseline updated. (T-1956, T-1957)
- [ ] gen-m7/cb748b6a9b81fbb7 A28, gen-m9/675763979c1defb6 A25 and gen-m9/adb3fd63c7368166 A34 pass, with no new
  known failures in eval/baseline/. (T-1956, T-1957)
- [x] ~~CHOOSE's `choose_array` is the kernel of a `lift_call`, with no result change.~~ Dropped by T-1959: CHOOSE stays off the driver (SPEC-0011's CHOOSE row).
- [x] The unprobed arguments are probed and recorded (probes/reference-lift-elements, lookup.md).
- [x] `make check` green; reviewers approve.

## Out of scope
- INDEX of a single value (T-1242).

## Progress (2026-10-19, opus)
Probe landed: probes/reference-lift-elements (excel-graph) answers the unprobed arguments (OFFSET cols, height,
width; INDIRECT a1; TRIMRANGE trim_cols; XMATCH search_mode) and the element shape. Findings are in lookup.md
"OFFSET, INDIRECT and TRIMRANGE lift into arrays of references". The key point for the design note: the elements
are references, not values. Spilled directly, each is `#VALUE!`; INDEX picks one as a reference (a multi-cell pick
spills or is `#SPILL!`); ROWS/COLUMNS count elements; SUM reads them. `ArrayNode` cannot hold that today, so the
design note has to choose a carrier (an array of references, or a lifted result INDEX/SUM/ROWS can see through)
before any code. No engine code yet; the new case's cells are recorded as known failures in eval/baseline.

## Outcome
Split on resumption (2026-10-19, opus): the probe showed the lifted elements are references that INDEX, ROWS and
SUM see through, which `ArrayNode` cannot carry. A new result carrier is an architectural change, which needs a
spec first (CLAUDE.md directive 4), so this task lands the evidence and hands the code to:
- T-1955 (engine, opus): spec for the array-of-references carrier.
- T-1956 (functions-lookup, opus): OFFSET on it. T-1957 (sonnet): INDIRECT and TRIMRANGE.
- T-1958 (functions-lookup, sonnet): XMATCH's search_mode lift, a value lift with no carrier needed.
- T-1959 (quality, opus): CHOOSE's index lift onto `lift_call`; decided against (see its Outcome).
Eval delta: probes +59 cells (3 pass, 56 new known failures in probes/reference-lift-elements, owned by T-1956 to T-1958);
0 regressions, 0 fixes.
