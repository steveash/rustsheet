---
id: T-3143
title: "Probe whether SINGLE/ANCHORARRAY, defined-name bodies, conditional formats and data validation count toward Excel's 65-call nesting cap"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for the corners of Excel's 65-call nesting cap that T-3097 left unprobed, and the engine matching it.

## Context
- T-3097 (docs/excel-semantics/syntax.md "Formula nesting depth cap") found Excel takes 65 nested function calls and
  refuses 66, stored (createSession 501) and typed (Graph PATCH 400). The engine counts calls in
  `Parser::in_call` (expressions/parser/depth.rs) and checks typed cell formulas (`typed_refusal`) and imported cell
  formulas (`from_a1_to_rc`, rustsheet-xlsx).
- Unprobed and so left unchecked: whether `@` stored as `_xlfn.SINGLE(` and `#` stored as `_xlfn.ANCHORARRAY(` count
  as levels (lists.rs `parse_function_args` exempts them); whether a defined-name body, a conditional-format formula
  or a data-validation formula past 65 calls makes Excel refuse the workbook; `Input::StoredFormula` through the API.
- Probe at 65 and 66 calls, with SINGLE/ANCHORARRAY at the innermost level and as one of the levels. Refused inputs
  are dropped after the run, as T-1091 and T-3097 did.

## Acceptance criteria
- [x] An excel-graph probe (milestone M2) for each form above; syntax.md records the result.
- [x] Where Excel refuses what the engine accepts (or the reverse), the counting or the import check is changed to
      match, with unit tests at 65 and 66.
- [x] `make check` green; reviewers approve.

## Out of scope
- The depth bound for parentheses (the DEVIATION in syntax.md).

## Outcome
- Probed (excel-graph 2026-10-30, probes/t3143-*): stored `_xlfn.SINGLE(` and `_xlfn.ANCHORARRAY(` count as calls
  (65 with one loads, 66 refuses the workbook); typed `@` and `#` operators add no level; typed `SINGLE(` spelled as a
  call counts (64+1 accepted, 65+1 refused); typed `ANCHORARRAY(` spelled as a call is refused by Graph at any depth
  (follow-up T-3262); a defined-name body of 66 nested calls refuses the workbook, 65 loads. syntax.md records it all.
- Engine: the parser no longer exempts spelled `SINGLE(`/`ANCHORARRAY(` from the call count (lists.rs); the importer
  refuses a name body nesting more than 65 calls (`reparse_formula_hack`). Unit tests at 65 and 66 in
  rustsheet-xlsx `malformed_import.rs` and rustsheet `typed_entry.rs`. `Input::StoredFormula` stays unchecked
  (documented; nothing in Excel to probe per cell).
- Not done: conditional-format and data-validation formulas need case-format support first (follow-up T-3263).
- Eval delta: +11 probe cases (22 cells), all bit-exact; no known failures added or removed.
