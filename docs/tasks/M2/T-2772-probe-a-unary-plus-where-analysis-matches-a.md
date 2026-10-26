---
id: T-2772
title: "Probe a unary plus where analysis matches a literal: N(IF(+TRUE,B1:B3)), N(CHOOSE(+TRUE,Z1,{2;3})), INDIRECT(+K1), legacy ABS(+{5}) and N(+{2;3}), LET(+x,1,x), a stored <f>+5</f>"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for the shapes where a kept unary plus (T-2723) may change a static reading: what Excel does with a
plus on a literal or name that an analysis matches by its kind.

## Context
T-2723 keeps a unary plus as `Node::UnaryPlus`. Before it, the parser dropped every plus. `ungrouped`, `lowered`,
the walkers, `is_plain_reference`'s literal test, CHOOSE's literal index test and `is_scalar_node` see through it.
Some sites match the raw node and do not, so they now read these shapes differently from main (Excel-fidelity
review of T-2723):
- `N(IF(+TRUE,B1:B3))` and `N(CHOOSE(+TRUE,Z1,{2;3}))` in a legacy formula, both before and after the T-2723 fix.
- `TAKE(IF(INDIRECT(+"K1"),…),2)`: the INDIRECT text check (static_analysis/aggregate.rs).
- Legacy `ABS(+{5})` (the `@{5}`) and `N(+{2;3})` (`for_each_value_operand` now gives `+@{2;3}`).
- `=LET(+x,1,x)`, and a LAMBDA parameter `+x`: Excel likely refuses them; we now refuse too (let.rs `NamedVariableKind`).
- A stored `<f>+5</f>` loaded from a file: we fold it to `5` on load, as typing does. All T-2723 evidence is typed
  (Graph PATCH).
Probe each on excel-graph (excel-probe skill): the values and FORMULATEXT, legacy-loaded where noted. Record the
results in syntax.md under "A typed unary plus stays in the formula text". File engine tasks for any mismatch.

## Acceptance criteria
- [ ] A probe case (excel-graph, oracle-filled) covers every shape above. Its failing cells are baselined.
- [ ] syntax.md records the results and names the engine tasks filed for any mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing fixes (filed as engine tasks).
