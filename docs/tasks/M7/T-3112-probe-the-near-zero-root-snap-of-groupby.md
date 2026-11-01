---
id: T-3112
title: "Probe the near-zero root snap of GROUPBY and PIVOTBY aggregation LAMBDAs and of eta-reduced helper calls"
milestone: M7
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
Excel evidence for whether GROUPBY and PIVOTBY snap a near-zero `+`/`-` root of their aggregation LAMBDA per group, and
whether an eta-reduced helper call (`MAP(r,SUM)`, `BYROW(r,SUM)`) snaps anything, so the engine can match each cell.

## Context
- Decision 0188 (owner's 2026-10-31 rulings, R1(c)): this task owns probe-row or text cells of its milestone's exit,
  which Phase 1 still counts in full, so it is in the semantics stage at P1 (was P3).
- T-3095 (probes/t3095-snap-lambda-helpers, excel-graph 2026-10-29) found that MAP, MAKEARRAY, BYROW, BYCOL, REDUCE
  and SCAN snap their LAMBDA body's root at every call (docs/excel-semantics/numbers.md "Near-zero snapping",
  paragraph "Pickers, LET, LAMBDA and unary signs"). Its out-of-scope note asks for this probe once per-call snapping
  is found.
- Copy the layout of eval/suites/probes/cases/t3095-snap-lambda-helpers.case.json. Use the bodies `x+0.2-0.3` at 0.1,
  `x+7*2^-52-1` and `x+8*2^-52-1` at 1 (k = 7 snaps and k = 8 keeps 1.7763568394002505e-15), and `-x-0.2+0.3` at 0.1.
  The in-place call `=LAMBDA(x,<body>)(v)` is the control.
- GROUPBY/PIVOTBY: give the function argument as `LAMBDA(v,SUM(v)-0.3)` over groups whose values sum to
  0.30000000000000004 (0.1 and 0.2), and use the k-forms with `-1`. Read the group rows and the total row through
  `INDEX`. The total row matters because it aggregates the values again rather than adding the group results.
- Eta-reduced: `MAP({0.1;0.2},SUM)` has no `+`/`-` root, so it is the control that nothing snaps. Also probe
  `BYROW(range,SUM)` against `BYROW(range,LAMBDA(r,SUM(r)-0.3))`.

## Acceptance criteria
- [ ] A new excel-graph probe case `probes/t3112-snap-groupby-eta` (milestone M7) reads back those cells.
- [ ] numbers.md records per form whether the aggregation call's root is snapped, citing the cells.
- [ ] Every cell the engine does not match is a known failure in eval/baseline/probes.txt, and one follow-up task
      (milestone M7, lane engine) names those cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the follow-up owns them; T-3111 owns the six helpers).
