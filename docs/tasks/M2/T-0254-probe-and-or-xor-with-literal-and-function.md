---
id: T-0254
title: "Probe AND/OR/XOR with literal and function-returned text arguments (1, abc, TRUE, empty text)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how AND, OR and XOR treat a text argument that is not a reference, so T-0255 can
implement it without guessing.

## Context
Desktop Excel cached `AND("1",1)` = `#VALUE!` (poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J136,
known failure; `AND(E14,E17,"1")` in I136 is also `#VALUE!`). The engine (`logical_nary` in
`crates/rustsheet-engine/src/functions/logical/and_or_xor_not.rs`) ignores a literal string unless it
reads as TRUE/FALSE, so it returns TRUE for J136. OR and XOR share that code but have no evidence, and
neither has a text value returned by a function (IFERROR, CONCATENATE, IF). Recorded so far in
docs/excel-semantics/coercion.md, "AND: a literal text argument that is not TRUE/FALSE is #VALUE!".

Probe case `eval/suites/probes/logical-text-args.case.json` (excel-probe skill), one cell each for AND,
OR and XOR:
- literal text: `("1",1)`, `("abc",TRUE)`, `("",TRUE)`, `(" ",TRUE)`, `("TRUE",1)`, `("true",0)`,
  `("FALSE",1)`, `("0",1)`, and `("abc")` alone;
- function-returned text: `(IFERROR("abc",0),TRUE)`, `(CONCATENATE("TRU","E"),0)`, `(IF(TRUE,"x"),1)`,
  `(T("TRUE"),0)`;
- text inside an array constant and a range (control: ignored today), `({"abc",TRUE})`, `(A1:A2,TRUE)`
  with A1 = "abc";
- `NOT("1")`, `NOT("abc")`, `NOT("TRUE")` for comparison.

## Acceptance criteria
- [ ] The probe case and its `.oracle.json` sidecar are in eval/suites/probes, oracle-filled.
- [ ] docs/excel-semantics/coercion.md entry rewritten from the evidence (VERIFIED per function and
      per argument kind; any difference between AND, OR and XOR recorded as such).
- [ ] Engine-side failures of the new case are baselined (known failures for T-0255), with no other
      baseline change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-0255).
- Which error wins among several error arguments (T-0229).
