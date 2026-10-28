---
id: T-3049
title: "Probe the near-zero snap of a root + or - under IF, IFS, IFERROR, CHOOSE, SWITCH, LET and a unary minus"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for whether a `+`/`-` that is not the formula's outermost node but is the value the formula returns
through a picker (IF, IFS, IFERROR, CHOOSE, SWITCH, LET, a LAMBDA call) or under a unary minus snaps a near-zero
result to 0, so the engine's notion of the "root" sum is backed by cells.

## Context
- docs/excel-semantics/numbers.md "Near-zero snapping": only the outermost `+`/`-` of a cell formula snaps; a
  written group (`=(0.1+0.2-0.3)`) and a group under an outer `+` do not (probes/near-zero-snap-groups), and a
  defined-name body snaps at its own root (T-1160). The engine decides this in `Model::evaluate_formula_root`
  (crates/rustsheet-engine/src/arithmetic.rs: `snaps = matches!(node, Node::OpSumKind { .. })`), so
  `=IF(TRUE,0.1+0.2-0.3)`, `=LET(x,0.1+0.2-0.3,x)`, `=IFERROR(0.1+0.2-0.3,0)` and `=-(0.1+0.2-0.3)` return the
  5.551115123125783e-17 residue today. No probe case holds any of these shapes (checked against the five near-zero
  case files and precision-and-rounding).
- Excel's rule may follow the last operator it executes rather than the tree's root (an IF evaluates its branch
  last), so the answer is not predictable from the group evidence. Wrong either way is visibly off.
- Opus: the evidence is unclear, and the boundary must be confirmed under each picker (k = 7 snaps and k = 8 does
  not for addends near 1, T-0110), which needs boundary pairs, not one sample.

## Acceptance criteria
- [ ] A new excel-graph probe case `probes/t3049-snap-under-pickers` (milestone M2, decision 0043) reads back
      `0.1+0.2-0.3` and a k = 7 / k = 8 boundary pair under each of: `IF(TRUE,..)`, `IF(FALSE,0,..)`, `IFS(TRUE,..)`,
      `IFERROR(..,0)`, `IFNA(..,0)`, `CHOOSE(1,..)`, `SWITCH(1,1,..)`, `LET(x,..,x)`, `LET(x,0.1,x+0.2-0.3)`,
      `LAMBDA(x,x+0.2-0.3)(0.1)`, a unary minus (`=-(0.1+0.2-0.3)` and `=-0.1-0.2+0.3`), and a cell reference to a
      snapped cell under `+0` as the control.
- [ ] numbers.md "Near-zero snapping" records which shapes snap, citing the cells, and states the rule the evidence
      supports (tree root, last executed operator, or another).
- [ ] Cells the engine does not match are known failures in eval/baseline/probes.txt with one filed engine follow-up
      task (milestone M2, tier opus) naming them; `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the follow-up task owns them).
- Array operands of the root `+`/`-` (T-3047) and the summing functions (T-3048).
