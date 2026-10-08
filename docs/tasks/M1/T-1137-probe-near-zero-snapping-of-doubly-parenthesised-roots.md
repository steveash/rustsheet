---
id: T-1137
title: "Probe near-zero snapping of doubly parenthesised roots and defined-name bodies"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles whether Excel snaps a near-zero final addition when the formula is wrapped in two or more levels of parentheses, and when the subtraction sits in a defined-name body, so docs/excel-semantics/numbers.md "Near-zero snapping" has no open Unprobed line for these forms.

## Context
M1 scope "Excel floating point: near-zero snapping". numbers.md "Near-zero snapping" (VERIFIED for `=0.1+0.2-0.3`, `=(0.1+0.2)-0.3`, `=SUM(0.1,0.2,-0.3)`, probes/precision-and-rounding A2-A4) says: "Unprobed: whether Excel also skips snapping for `=((X-b))` ... and inside a defined-name body; the engine marks the first two as grouped roots and never the third (name bodies are parsed with `Parser::parse`)". The single-level grouped root `=(X-b)` is not snapped (T-1017, `Node::GroupedRootKind`, `Parser::parse_cell_formula`). The engine's choices for the deeper and name-body forms are guesses. A formula typed as `=(X-b` that Excel completes cannot be written through the xlsx path, so it is not part of this task. SUM forms belong to T-1131; do not repeat them here.

Follow the `excel-probe` skill. Add one new case, probes/near-zero-snap-groups (`names` holds the defined names, as in probes/union-defined-name), with at least these cells:
- `=((0.1+0.2-0.3))`, `=(((0.1+0.2-0.3)))`, `=((1-0.9-0.1))`, and FORMULATEXT of the first (numbers.md notes the engine stores `((A1-1))` as one level, a known gap).
- `=(0.1+0.2-0.3)+0` and `=0+(0.1+0.2-0.3)` (a group that is an operand of an outer `+`).
- A name `nz` with body `=0.1+0.2-0.3`, and a name `nz2` with body `=Sheet1!$A$1-0.3` (A1 = `=0.1+0.2`), used as `=nz`, `=nz2`, `=nz+0`, `=nz*1` and `=SUM(nz)`.
- Controls: `=0.1+0.2-0.3` and `=(0.1+0.2-0.3)` (the known results).

## Acceptance criteria
- [ ] probes/near-zero-snap-groups is committed with an excel-graph oracle sidecar covering every cell listed above; the oracle values are canary-verified as the excel-probe skill requires.
- [ ] numbers.md "Near-zero snapping" records the result for each form with the case id and oracle date, and its Unprobed sentence no longer lists `=((X-b))` or the defined-name body.
- [ ] Cells the engine gets wrong are recorded with `make eval-baseline` as new-coverage failures (no other F line added), and one follow-up engine task owning them is filed and named in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `snap_near_zero`, `evaluate_formula_root` or the parser (file the follow-up instead).
- SUM forms (T-1131) and array operands.
- Typed (unclosed-parenthesis) formula entry.
