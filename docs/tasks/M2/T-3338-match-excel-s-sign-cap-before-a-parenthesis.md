---
id: T-3338
title: "Match Excel's sign cap before a parenthesis or call (1,022) and let -( nest to the 256-parenthesis cap (t3266 probes)"
milestone: M2
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
RustSheet accepts and refuses the same sign runs and `-(` nestings as Excel. The rule is in
docs/excel-semantics/syntax.md "Formula nesting depth cap", in the T-3266 paragraph.

## Context
- Excel's rule (T-3266, excel-graph 2026-10-31): the signs open at an operand, counted across nested
  runs, plus that operand's cost may be at most 1,024. A number literal right after the signs costs 0;
  a reference, `TRUE`, a string or a number after a space costs 1; a parenthesis run or a function call
  costs 2.
- Ours (`MAX_SIGNS` 1,023 in crates/rustsheet-engine/src/expressions/parser/depth.rs,
  `Parser::parse_signs`) counts every sign except a minus directly before a number literal. That
  matches Excel before literals, references, booleans and strings.
  - Before `(` or a call, ours admits 1,023 signs where Excel admits 1,022. Excel refuses
    `"-"*1023 & "(1)"`, `"-"*1023 & "((1))"`, `"+"*1023 & "(1)"` and `"-"*1023 & "ABS(1)"`, stored and
    typed. The stored ones keep no sidecar, so pin them in parser unit tests (test_nesting_depth.rs);
    the typed ones are probes whose sidecar records the refusal (A1 blank, `ISFORMULA` FALSE).
  - Ours bounds `-(` nesting by `MAX_FORMULA_DEPTH`: each group holding `-(...)` costs an operand's
    two levels, so 128 levels are too deep. Excel accepts `-(` up to 256 levels (the parenthesis cap)
    and refuses 257.
- Directive 5: deeper `-(` nesting must stay safe for the evaluator, stringifier and any recursive
  walk. Each walk must be iterative or explicitly bounded; check
  expressions/parser/stringify/wrapped.rs, which already claims "`-(-(...))` nests as deep as
  `((...))`", and the evaluator's recursion budget. A Rust-stack test at 256 levels is required.
- Failing cells (Excel accepts, ours refuses with EngineError):
  - probes/t3266-nested-paren-128-stored.case.json, probes/t3266-nested-paren-128-typed.case.json,
    probes/t3266-nested-paren-200-stored.case.json, probes/t3266-nested-paren-200-typed.case.json,
    probes/t3266-nested-paren-255-stored.case.json, probes/t3266-nested-paren-255-typed.case.json,
    probes/t3266-nested-paren-256-stored.case.json, probes/t3266-nested-paren-256-typed.case.json,
    probes/t3266-nested-200-823-stored.case.json, probes/t3266-nested-200-823-typed.case.json,
    probes/t3266-nested-200-824-stored.case.json, probes/t3266-nested-200-824-typed.case.json
    (each whole case: today they do not run at all, EngineError "Formula is nested too deeply")
  - Excel refuses, ours accepts: probes/t3266-minus-paren-1023-typed.case.json,
    probes/t3266-minus-paren2-1023-typed.case.json, probes/t3266-plus-paren-1023-typed.case.json,
    probes/t3266-minus-abs-1023-typed.case.json: S!B1

## Acceptance criteria
- [ ] Every probes/t3266-* case passes; `make eval-baseline` records them with 0 regressions.
- [ ] Parser unit tests pin these limits: 1,022 accepted and 1,023 refused before `(1)`, `((1))` and
      `ABS(1)`; 257 levels of `-(` refused.
- [ ] A 256-level `-(` formula parses, evaluates, stringifies and round-trips xlsx without stack
      overflow (a test on a small thread stack, as the existing depth tests do).
- [ ] syntax.md "Ours differs" sentence removed; `make check` green; reviewers approve.

## Out of scope
- Other wrappers around sign runs, such as `ABS(-ABS(-...))`, which the 65-call cap already bounds.
