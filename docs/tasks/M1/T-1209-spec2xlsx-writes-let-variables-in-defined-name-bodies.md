---
id: T-1209
title: "spec2xlsx writes LET variables in defined-name bodies without _xlpm. (follow-up of T-1207)"
milestone: M1
lane: harness
status: todo
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
- [ ] A LET name probe returns the LET result in Excel, not `#NAME?`; tools tests cover the prefixing of name bodies.
- [ ] Existing oracle sidecars still match their built workbooks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
