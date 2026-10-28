---
id: T-2938
title: "Typed SINGLE(x): probe whether Excel takes it as the @ operator and match it (probes/t2576-error-type-cell-paired D27, ours #NAME?)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A formula that calls `SINGLE(x)` by name, typed or set through the API, evaluates as Excel evaluates it. If Excel takes a
typed `=SINGLE(x)` as the implicit-intersection operator `@x`, ours does too, so
probes/t2576-error-type-cell-paired Calc!D27 `=FTEST({1;2;3},SINGLE(ERROR.TYPE(#REF!)))` is `#DIV/0!` (Excel) and not
`#NAME?` (ours). If Excel refuses the typed name, that rule is recorded and the harness follow-up is filed (see Context).

## Context
- Failing cell: probes/t2576-error-type-cell-paired Calc!D27 (excel-graph, T-2576). Its verdict is `unsupported`, the
  only `unsupported` cell in the 2026-10-27 report. T-2766 (M9) lists D27 and says to "move it to the task that owns
  SINGLE if one exists". No open task owns SINGLE, so this task does. D27 stays listed in T-2766 for the M9 exit.
- Why the two sides differ: tools/src/rustsheet_tools/spec2xlsx.py `_BUILDER_ONLY_PREFIXES` rewrites `SINGLE(` to
  `_xlfn.SINGLE(`, so Excel got the stored form of `@`. The engine adapter passes the spec text unchanged as
  `Input::StoredFormula` (crates/rustsheet-eval/src/engine.rs `spec_input`, decision 0103). The parser only maps
  `_xlfn.SINGLE` to `Node::at` (crates/rustsheet-engine/src/expressions/parser/operands.rs, the "only importing functions
  from xlsx" branch near line 534). A bare `SINGLE` is an unknown function, so we give `#NAME?`.
- Which side is right depends on Excel's typed entry, and no evidence covers it. probes/intersection-single-operands
  (T-1103) and docs/excel-semantics/xlsx-storage.md ("`@(x)` is `_xlfn.SINGLE((x))`", T-1343) cover the stored form only.
  Typed entry can be probed on excel-graph: an `{"entered": "=..."}` cell is typed through Graph `values`, and Excel's
  refusal comes back as a 400 `InvalidArgument` (T-2040, T-2497, decision 0103). Put each entry that may be refused in a
  case of its own, because a refused entry fails the whole fill.
- Probe first (excel-probe skill), in a new M2-tagged probe case. Entries `=SINGLE(B1:B3)` (beside rows 1-3 and off
  them), `=SINGLE(5)`, `=SINGLE({1,2})`, `=SINGLE(ERROR.TYPE(#REF!))`, `=single(B1:B3)` (lower case), `=SINGLE()`,
  `=SINGLE(1,2)`, each with `=FORMULATEXT(..)` beside it to show whether Excel stores `@`. Then match:
  - Excel takes it as `@`: a bare `SINGLE(x)` with one argument parses to the same `@` node in typed and API input, and
    FORMULATEXT prints what Excel printed. Wrong argument counts give what Excel gives.
  - Excel refuses it or gives `#NAME?`: ours keeps `#NAME?` (or refuses typed input as decision 0103 does), and the
  builder's `SINGLE` rewrite misstates the spec. Record that and file a harness task to stop rewriting a typed `SINGLE(`
  and refill t2576 D27. That task then owns D27.
- Leave the stored-file form alone. A bare `SINGLE(` read from an xlsx follows T-1089's bare-newer-name rule.

## Acceptance criteria
- [x] A new probe case with the typed entries above is oracle-filled (excel-graph) and committed, with refused entries
      recorded in the semantics note. The rule goes into docs/excel-semantics/xlsx-storage.md (or the `@` section it
      links to), with Status and Evidence.
- [x] If Excel takes typed `SINGLE` as `@`: probes/t2576-error-type-cell-paired Calc!D27 passes and leaves
      eval/baseline/probes.txt, the new case passes, and probes/intersection-single-operands keeps passing. A unit test
      in the parser tests covers `SINGLE(B1:B3)` typed, and its FORMULATEXT if Excel showed one.
- [x] (n/a: Excel takes it) If Excel refuses it: the harness follow-up is filed and names D27, and the engine keeps `#NAME?`.
- [x] `make eval-baseline` shows no regression; `make check` green; reviewers approve.

## Out of scope
- Bare `SINGLE(` and other bare newer names read from an xlsx file (T-1089).
- The other builder-only prefixes (ANCHORARRAY, FILTERXML, IMAGE, QUERYSTRING, WEBSERVICE).
- The ERROR.TYPE mark of the paired functions (T-2766).

## Outcome
Excel takes a typed `SINGLE(x)`, in any case, as `@x`, and FORMULATEXT shows `@x` (SINGLE's parentheses dropped):
probes/single-typed-01 to -06 and -09 (excel-graph, 2026-10-27), with one typed entry per case. Typed `=SINGLE()` and
`=SINGLE(1,2)` were refused (Graph 400), so cases -07 and -08 were dropped, and that is recorded in xlsx-storage.md
"Typed `SINGLE(x)` is the `@` operator". The builder's `SINGLE(` to `_xlfn.SINGLE(` rewrite is therefore faithful, and no
harness follow-up is needed. The parser (operands.rs `parse_named_call`) reads a bare `SINGLE(` with one argument as the
`@` node in typed, API and stored text. A typed call with another argument count is refused like other wrong counts
(`wrong_count_call`, decision 0103). Unit tests are in test_implicit_intersection.rs. Eval delta: probes/t2576-error-type-cell-paired
Calc!D27 is fixed and leaves eval/baseline/probes.txt; there are 7 new passing cases (14 cells); probes/intersection-single-operands
still passes. D27 also leaves T-2766's list.
