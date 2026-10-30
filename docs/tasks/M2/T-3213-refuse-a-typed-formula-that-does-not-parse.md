---
id: T-3213
title: "Refuse a typed formula that does not parse (Input::Parse/Formula), as Excel's PATCH 400 does, instead of storing #ERROR!"
milestone: M2
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A formula typed through the facade (`Input::Parse` / `Input::Formula`) that does not parse is refused with `EditError::InvalidInput` and leaves the cell as it was, as Excel refuses it (Graph PATCH 400 `InvalidArgument`), instead of being stored as a parse-error formula (`#ERROR!`). Stored text (`Input::StoredFormula`, xlsx import) keeps today's parse-error cell.

## Context
Evidence: probes/t3113-entered-refused S!B1 (`{"entered": "=1+"}`, `=ISBLANK(A1)` TRUE on excel-graph; the engine
gives FALSE, the cell holds `#ERROR!`), a known failure in eval/baseline. The eval adapter (crates/rustsheet-eval/src/engine.rs, the `tx.set` of a spec cell) fails the whole case on
`InvalidInput`, which the resource-limit refusal deviations rely on (probes/t3172-*-typed, t3097-*-typed); a refused
parse error must leave the cell blank instead (decision 0179), so tell the two apart (a distinct error, or match the
message). docs/excel-semantics/references.md says "a typed one stays a parse
error (`#ERROR!`, the text kept)" in two places (structured references to a missing table, `[1]!Table3[#Data]`):
check those against Excel (probe first) and update them. Code: `Model::set_typed_user_input`
(crates/rustsheet-engine/src/typed_input.rs) already refuses decision-0103 forms via `Parser::typed_refusal`; a
parse-error node can join it. Watch the Python bindings and facade tests that expect a typed parse error to store.

## Acceptance criteria
- [x] probes/t3113-entered-refused passes (S!B1 off the baseline); more probe shapes (unbalanced parenthesis, unknown operator) agree.
- [x] Stored parse errors (xlsx import, `Input::StoredFormula`) unchanged.
- [x] excel-semantics entries updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A typed formula (`Input::Parse`/`Formula`, `Model::set_typed_user_input`) that does not parse is refused with an
error starting `TYPED_FORMULA_DOES_NOT_PARSE`, and the cell keeps its content. The `)`-retry still closes a call
(`=SUM(1`), but not a group (`=(1`), as Excel does. Stored text is unchanged. The eval adapter leaves a refused
typed cell blank (decision 0179). New probe t3213-typed-parse-errors (11 shapes, excel-graph): 8 refused, 2
auto-closed calls, and `=1**2` stores `=100` in Excel (follow-up T-3283). Eval: fixed 2 (t3113-entered-refused
S!B1 and one more), new case with 3 known failures (`=1**2`), and one accepted regression, t3003 S!F2, a coincidental
pass now owned by T-3274 (decision 0186). references.md's two "typed stays a parse error" claims now cite the probe.
