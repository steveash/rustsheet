---
id: T-3689
title: "Score typed refusals Excel also makes (257 parentheses, 66 nested calls) as blank cells and restore the dropped typed-refusal probes"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A typed formula that Excel refuses (Graph PATCH 400 `InvalidArgument`, cell left blank) is refused by the API with
`TYPED_FORMULA_DOES_NOT_PARSE`, so the harness scores it as the blank cell Excel leaves, for the parenthesis cap and
the call cap as T-3338 did for the sign cap. The dropped typed-refusal probes are back and pass.

## Context
- T-3338 made the sign cap a non-limit parse error (`TOO_MANY_SIGNS_MESSAGE`, expressions/parser/depth.rs), so
  `typed_input.rs` reports it as not parsing and the eval adapter (crates/rustsheet-eval/src/engine.rs, decision 0179)
  keeps the cell blank. The 257-parenthesis refusal (`TOO_DEEP_MESSAGE` from `MAX_PARENTHESES`) and the 66-call refusal
  (`Parser::typed_refusal`, `MAX_NESTED_CALLS`) are still errors the harness scores as `EngineError` (decision 0168).
- Excel refuses both typed with PATCH 400 and a blank A1 (docs/excel-semantics/syntax.md "Formula nesting depth cap":
  probes/t3172-paren-257, t3266-nested-paren-257, t3097-nesting-*-66).
- The typed refusals T-3266 and T-3172 dropped because of this can be restored: the 1,024/1,025 sign runs,
  t3266-nested-200-825-typed, t3266-nested-50-975-typed, t3266-nested-paren-257-typed (their sidecars: no A1,
  `ISFORMULA(A1)` FALSE). The sign-cap ones need only the oracle files back, since T-3338.
- Keep `MAX_FORMULA_DEPTH` refusals that Excel would accept as limit errors (directive 5, decision 0014): only a
  refusal Excel also makes may become "does not parse".

## Acceptance criteria
- [ ] Typed past the parenthesis cap or the call cap, `set_typed_user_input` errs with `TYPED_FORMULA_DOES_NOT_PARSE`
      and the cell keeps its content; unit tests pin both.
- [ ] The dropped typed-refusal probes are restored (re-run the oracle where a sidecar is gone) and pass.
- [ ] syntax.md updated; `make check` green; reviewers approve.

## Out of scope
- Stored refusals (Excel refuses the workbook; they keep no sidecar).
