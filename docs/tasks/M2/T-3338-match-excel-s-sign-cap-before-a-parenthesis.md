---
id: T-3338
title: "Match Excel's sign cap before a parenthesis or call (1,022) and let -( nest to the 256-parenthesis cap (t3266 probes)"
milestone: M2
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
RustSheet accepts and refuses the same sign runs and `-(` nestings as Excel. The rule is in
docs/excel-semantics/syntax.md "Formula nesting depth cap", in the T-3266 paragraph.

## Context
- Raised to P1 (2026-11-01): the 12 t3266 probe cases that do not run (engine_error) make M2's Phase 1 exit probe row "not met" by themselves (decision 0174).
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
    probes/t3266-nested-200-824-stored.case.json, probes/t3266-nested-200-824-typed.case.json: S!A1, S!B1
  - The whole of probes/t3266-nested-200-824-stored and probes/t3266-nested-200-824-typed: neither case runs today
    (EngineError "Formula is nested too deeply"), so the owner check reads them as one failing item each, which a
    line naming their cells does not cover (gap triage 2026-10-31).
  - Excel refuses, ours accepts: probes/t3266-minus-paren-1023-typed.case.json,
    probes/t3266-minus-paren2-1023-typed.case.json, probes/t3266-plus-paren-1023-typed.case.json,
    probes/t3266-minus-abs-1023-typed.case.json: S!B1

## Acceptance criteria
- [x] Every probes/t3266-* case passes; `make eval-baseline` records them with 0 regressions.
- [x] Parser unit tests pin these limits: 1,022 accepted and 1,023 refused before `(1)`, `((1))` and
      `ABS(1)`; 257 levels of `-(` refused.
- [x] A 256-level `-(` formula parses, evaluates, stringifies and round-trips xlsx without stack
      overflow (a test on a small thread stack, as the existing depth tests do).
- [ ] syntax.md "Ours differs" sentence removed; `make check` green; reviewers approve. (Sentence
      removed and `make check` green; reviewers not yet run.)

## Out of scope
- Other wrappers around sign runs, such as `ABS(-ABS(-...))`, which the 65-call cap already bounds.

## Outcome
Root cause: two parser rules. (1) The sign cap counted every sign but a minus directly before a
number against 1,023, which is Excel's rule only for operands costing 0 or 1; a parenthesis or a call
costs 2. (2) Each level of `-(` was a recursive operand costing two levels of `MAX_FORMULA_DEPTH`, so
128 levels were "nested too deeply" (`EngineError`), though Excel nests them to the 256-parenthesis cap.

Fix:
- `MAX_SIGNS` is now Excel's rule as written: open signs plus the operand's cost at most 1,024 (number
  literal right after the signs 0, parenthesis or call 2, else 1; `Parser::parse_signs`,
  `Parser::parse_signed_operand`). Past it the error is `TOO_MANY_SIGNS_MESSAGE`, not a resource limit,
  so a typed formula is refused as not parsing and the cell stays blank, as Excel's PATCH 400 leaves it.
  That is what lets the four 1,023-sign typed probes score.
- `Parser::parse_parenthesis_run` reads signs between the `(`s of a run in its loop. Each level's signs
  are handed to the enclosing content with the pending group (`pending_prefix`). A group whose content is
  exactly a signed group adds no depth, so `-(` nests to `MAX_PARENTHESES` with constant parser stack.
- `Model::apply_unary_chain` walks through groups too, so evaluating `-(` takes one frame at any
  depth. It counts the minus parity per group, so the tiny-subnormal `-x` rule applies as it did when
  each group was evaluated alone. The printer (`wrapped_text`) was already a loop.
- Directive 5: these tests pass. Bare `-(` and `+(` at 256 levels on a 2 MiB debug stack.
  Spaced `- ( `, 200 levels plus 824 signs, and `-(...-B1%)` on 1 MiB release / 16 MiB debug, the
  sign-cap tests' sizing (checked with `cargo test --release`). Each case parses, evaluates, prints,
  survives a row insert and a sheet rename, and round-trips xlsx
  (crates/rustsheet-xlsx/tests/signed_parentheses_round_trip.rs). The spaced form overflows 2 MiB
  unoptimized in the derived `Clone` of a stored tree, as the existing 1,023-sign spaced tests would;
  that is why those use the small-stack sizing. No DEVIATION is needed: every depth Excel allows here
  now loads.

Eval delta (probes, verified worktree binary): known failures 3,282 -> 3,266 (16 fixed; 0 regressions).
The 12 cases that did not run (nested-paren-128/-200/-255/-256 and nested-200-823/-824, stored and
typed) now run and pass 2/2 cells each, and the four typed 1,023-sign cases pass S!B1.
M2 probe row (M0-M2): before (eval of main b6c931bd, 2026-11-01) 198 known failures, 186 visibly off,
12 blocked cases, not met. After (this branch, full run) 182 known failures, 182 visibly off, 0 blocked
cases, still not met. The cells left are the collation and criteria probes (T-3324, T-3325, T-3329,
T-3404), none of them T-3266's.
