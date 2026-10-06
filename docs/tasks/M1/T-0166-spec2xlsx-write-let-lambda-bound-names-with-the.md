---
id: T-0166
title: "spec2xlsx: write LET/LAMBDA bound names with the _xlpm. prefix"
milestone: M1
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A spec case using LET or LAMBDA builds an xlsx Excel for the web loads, so such probes can be oracle-filled.

## Context
`rustsheet-spec2xlsx` (tools/src/rustsheet_tools/spec2xlsx.py) relies on XlsxWriter's future-function
prefixes, which add `_xlfn.LET` but leave the bound names bare. Excel stores them as `_xlpm.x`
(the engine's exporter does: expressions/parser/stringify/bound_variables.rs). In T-0161 the scratch
workbook `=A1:A3 _xlfn.LET(x,A2:B2,x)` was refused (`FileCorruptTryRepair`) while
`_xlfn.LET(_xlpm.x,A2:B2,_xlpm.x)` loaded and gave 2.

## Acceptance criteria
- [ ] LET and LAMBDA parameter names (and their uses) are written with `_xlpm.`; a unit test on the generated sheet XML.
- [ ] Functions XlsxWriter does not know get their `_xlfn.` prefix too: T-0161's `=A1:A3 TRIMRANGE(A2:B2)` came back `#NAME?` from the case builder but 2 with `_xlfn.TRIMRANGE` written by hand. Take the prefix list from the engine's exporter rather than a second copy.
- [ ] `=A1:A3 LET(x,A2:B2,x)` and `=A1:A3 TRIMRANGE(A2:B2)` added to probes/intersection-function-operands and oracle-filled (2 expected).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Generic formula translation beyond bound names.

## Outcome
spec2xlsx now writes LET/LAMBDA names as `_xlpm.x` and prefixes engine-known future functions
(list read from `Function::to_xlsx_string`, except the classic CEILING, which the exporter wrongly
writes as `_xlfn.CEILING`). Probes H13 (LET) and H14 (TRIMRANGE) added and oracle-filled (both 2).
Eval delta: probes known failures 103 -> 104 (H14: the engine's TRIMRANGE as an intersection
operand gives #VALUE!, like TAKE/DROP). Follow-up: the exporter's CEILING prefix (T-0172).
