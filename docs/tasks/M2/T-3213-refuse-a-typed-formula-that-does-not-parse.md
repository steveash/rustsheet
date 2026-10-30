---
id: T-3213
title: "Refuse a typed formula that does not parse (Input::Parse/Formula), as Excel's PATCH 400 does, instead of storing #ERROR!"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A formula typed through the facade (`Input::Parse` / `Input::Formula`) that does not parse is refused with `EditError::InvalidInput` and leaves the cell as it was, as Excel refuses it (Graph PATCH 400 `InvalidArgument`), instead of being stored as a parse-error formula (`#ERROR!`). Stored text (`Input::StoredFormula`, xlsx import) keeps today's parse-error cell.

## Context
Evidence: probes/t3113-entered-refused S!B1 (`{"entered": "=1+"}`, `=ISBLANK(A1)` TRUE on excel-graph; the engine
gives FALSE, the cell holds `#ERROR!`), a known failure in eval/baseline. The eval adapter already leaves a typed
entry blank on `InvalidInput` (decision 0179). docs/excel-semantics/references.md says "a typed one stays a parse
error (`#ERROR!`, the text kept)" in two places (structured references to a missing table, `[1]!Table3[#Data]`):
check those against Excel (probe first) and update them. Code: `Model::set_typed_user_input`
(crates/rustsheet-engine/src/typed_input.rs) already refuses decision-0103 forms via `Parser::typed_refusal`; a
parse-error node can join it. Watch the Python bindings and facade tests that expect a typed parse error to store.

## Acceptance criteria
- [ ] probes/t3113-entered-refused passes (S!B1 off the baseline); more probe shapes (unbalanced parenthesis, unknown operator) agree.
- [ ] Stored parse errors (xlsx import, `Input::StoredFormula`) unchanged.
- [ ] excel-semantics entries updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
