---
id: T-1209
title: "spec2xlsx writes LET variables in defined-name bodies without _xlpm. (follow-up of T-1207)"
milestone: M1
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A defined-name body that uses LET (or LAMBDA) is written the way Excel stores it, so probes can use LET in names.

## Context
`spec2xlsx.py` runs `_formula_text` (`prefix_bound_names`, `prefix_future_functions`) on cell formulas but passes names straight to `wb.define_name`. In probes/countblank-formula-names-b (T-1207, now without LET) `=LET(x,5,x)` as a name read back `#NAME?` in Excel: the variable was an undefined name. Apply the same prefixing to names, check that existing cases with future functions in names keep the same built bytes (or re-fill their sidecars), then add the LET-bodied COUNTBLANK names from T-1207 (`=LET(x,5,x)`, `=LET(x,1/0,x)`, `=LET(x,Sheet1!$B$1:$B$3,x)`, `=LET(x,Sheet1!$C$2,x)`).

## Acceptance criteria
- [x] A LET name probe returns the LET result in Excel, not `#NAME?`; tools tests cover the prefixing of name bodies.
- [x] Existing oracle sidecars still match their built workbooks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`spec2xlsx` now runs `_formula_text` on name bodies; test in `test_spec2xlsx_names.py`. Added four LET names to probes/countblank-formula-names-b (A14:A17, Excel: #VALUE!, #DIV/0!, 1, 1; engine passes all). Re-filled sidecars: countblank-formula-names-b and isref-names-filter-if-empty (LAMBDA name body now `_xlpm.x`; values unchanged). Eval: probes 17/17 cells on that case, no regressions.
