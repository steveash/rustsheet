---
id: T-3097
title: "Probe Excel's 64-level function nesting cap on load and when typed, and refuse deeper formulas as Excel does"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Oracle evidence for where Excel refuses nested function calls and nested parentheses, in a stored workbook and in a
typed formula, and an engine that accepts and refuses the same formulas: the xlsx importer refuses a workbook Excel
refuses, the API refuses a typed formula Excel refuses, and every formula Excel accepts still evaluates.

## Context
- docs/excel-semantics/syntax.md "Formula nesting depth cap": the engine refuses trees deeper than `MAX_FORMULA_DEPTH`
  (256; expressions/parser/depth.rs), so it accepts up to 127 nested calls or parentheses. Excel's published limit is
  64 nested levels of functions, and the entry says "RustSheet accepts 65 to 127 nested calls, which Excel refuses per
  its published limit (UNVERIFIED: no oracle probe yet)" and "Excel's paren-nesting cap is not measured". A workbook
  formula past the engine's cap loads as a parse error in that cell, while Excel refuses whole workbooks for other
  limits it enforces on load.
- Precedents for limits Excel enforces on load and when typed, and how to probe them:
  - syntax.md "Formulas over 8,192 characters refuse the workbook" (T-1091, T-1397): Excel for the web refuses the
    whole workbook (createSession 501 `FileCorruptTryRepair`); refused inputs cannot keep a sidecar, so those cases
    were dropped after the run and the result recorded in the doc, while the accepted boundary case was committed.
    The importer refuses the same workbooks with `XlsxError::Workbook` (`MAX_FORMULA_UNITS`,
    crates/rustsheet-xlsx/src/import/sheet_data.rs; decision 0047).
  - Typed formulas through the Graph PATCH path (T-2709, T-2739): a refused formula is a 400 `InvalidArgument`.
- Which levels count is open: Excel's limit is on functions, so `=((((1))))` may nest deeper than `=ABS(ABS(1))`, and
  whether an operator, an array constant, a LET/LAMBDA or a function's own parentheses add a level is unknown.
- Why opus: the probe needs one workbook per refused input, a design that tells "function levels" from "parentheses"
  from "tree depth", and the engine change touches the parser's depth accounting (depth.rs, lambda.rs, chain.rs),
  the importer and the API edit path, under directive 5 (the stack-depth bound must not grow past what the at-limit
  tests measure on a 1 MiB stack).

## Acceptance criteria
- [ ] Probed on excel-graph, stored and typed: `=ABS(ABS(...(1)...))` with 63, 64, 65 and 66 calls; the same with IF,
      SUM and a mixed chain; parentheses only (`=((...(1)...))`) at 64, 65, 127, 128 and 255 levels; calls inside
      parentheses (each level `(ABS(`); an operator between levels (`=ABS(1+ABS(1+...))`); and a LET or LAMBDA at the
      innermost level. Accepted boundary cases are committed as probe cases `probes/t3097-nesting-*` (milestone M2,
      decision 0043) whose value cells pass; refused inputs are recorded with their error and dropped, as T-1091 did.
- [ ] syntax.md "Formula nesting depth cap" states Excel's rule for each form with that evidence and drops the
      UNVERIFIED sentence.
- [ ] Where Excel refuses a stored formula the engine accepts, `rustsheet-xlsx` refuses the workbook with
      `XlsxError::Workbook`, and a typed formula Excel refuses is refused through the API with `EditError::InvalidInput`;
      unit tests cover one formula on each side of every boundary found. A form Excel accepts beyond the engine's
      depth bound stays refused and is listed in the entry as a DEVIATION (directive 5, decision 0014) with its
      boundary cells.
- [ ] No eval case changes value (`make eval-check`: 0 regressions); the at-limit parser tests still pass on a 1 MiB
      stack.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Raising `MAX_FORMULA_DEPTH` or `MAX_VALUE_OPERATORS` (a spec first, if the probe shows Excel accepts deeper trees).
- Defined-name bodies, conditional-format and data-validation formulas: file a follow-up if the rule applies to them.
- Desktop Excel's typed-formula dialog (M14).

## Progress (run 1, 2026-10-29; the task stays in progress)
- Probed on excel-graph (60 inputs; 44 committed as `probes/t3097-nesting-*`, stored and typed). Excel accepts 65 nested
  function calls and refuses 66 (stored: createSession 501; typed: 400 `InvalidArgument`) for every form: ABS, IF, SUM,
  a mixed chain, an operator between levels, calls inside parentheses, and LET or LAMBDA innermost. Parentheses add no
  level (255 load). The 16 refused 66-call inputs were dropped after the run. syntax.md "Formula nesting depth cap"
  records the rule, and the UNVERIFIED sentence is gone.
- The engine refuses 8 of the committed cases, all of which Excel accepts (paren-128, paren-255 and paren-abs-64/65,
  past `MAX_FORMULA_DEPTH`). They are new known failures and listed as the DEVIATION.
- Left: count function-call nesting in the parser (depth.rs), refuse 66+ calls on import (`XlsxError::Workbook`) and
  through the API (`EditError::InvalidInput`), with unit tests at 65 and 66 for each form.
