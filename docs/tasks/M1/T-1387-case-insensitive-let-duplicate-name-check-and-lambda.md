---
id: T-1387
title: "Duplicate LET/LAMBDA names that differ only in case (follow-up of T-1378)"
milestone: M1
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Establish, then make the engine match, what Excel does with duplicate names that differ only in case: `LET(x,1,X,2,x)`, `LAMBDA(x,X,x+X)(1,2)` (and non-ASCII names such as `LET(é,1,É+1)`).

## Context
T-1378 made LET names and LAMBDA parameters case-insensitive (`same_variable`, functions/logical/let.rs; probes/let-lambda-name-case). Excel for the web refused (`FileCorruptTryRepair`) the spec2xlsx workbooks holding the three forms above, so they are unprobed. `seen_names` in let.rs (~138), lambda.rs and parser/lambda.rs still compare case-sensitively. Find out whether the refusal is Excel rejecting the formula (then the engine should refuse it too, see syntax.md) or a spec2xlsx encoding problem (the `é` form suggests the latter); desktop Excel may be needed (M14, decision 0039).

## Acceptance criteria
- [x] A probe case with oracle values for the forms above; the engine matches it. Docs entry in references.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probes with `rustsheet-oracle recalc -b excel-graph` (single-cell workbooks, 2026-10-12): Excel for
  the web refuses (`FileCorruptTryRepair`) every LET or LAMBDA that binds one name twice, in any
  case and including non-ASCII names: `LET(x,1,x,2,x)`, `LET(x,1,X,2,x)`, `LET(é,1,É,2,é)`,
  `LAMBDA(x,x,x)(1,2)`, `LAMBDA(x,X,x+X)(1,2)` and `LAMBDA(ä,Ä,ä+Ä)(1,2)`. The engine now gives
  `#ERROR!` for all of them: the LET duplicate check is case-insensitive, and a LAMBDA with
  duplicate parameters is a parse error (`has_duplicate_parameter`).
- The refusal of `LET(é,1,É+1)` was spec2xlsx leaving a non-ASCII name without its `_xlpm.`
  prefix (ASCII-only `_IDENT`). With that fixed, it loads and gives 2. The new case
  probes/let-name-non-ascii (10 cells, oracle: excel-graph) records Unicode case folding.
- The engine rejected a name whose first letter is non-ASCII (`is_valid_a1_identifier` was
  ASCII-only, apart from letters such as `ß`, which uppercases to `SS`); it now accepts a letter
  of any script.
- The xlsx-import path (`fn_lambda`, for a stored `FunctionKind::Lambda`) shares the parser's
  check, `has_duplicate_parameter`. More probes: `LAMBDA(x,[X],x)(1)` is refused as well, and
  probes/let-name-sharp-s (2 cells) shows `ß` is `ẞ` but not `SS` (`#NAME?`).
- docs/excel-semantics/references.md is updated. Follow-up T-1428: CJK, Turkish İ and non-ASCII
  defined names.
- Eval delta: +2 probe cases (12 cells, all passing); known failures unchanged (2874).
