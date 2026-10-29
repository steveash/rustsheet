---
id: T-3140
title: "Near-zero snap of a root + or - over coerced operands (number, percent, time and date text, booleans, blank cells): probe and match"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Oracle evidence settles whether the root `+`/`-` near-zero snap applies, with the same 8-ULP bound, when an addend
is a coerced value rather than a number. The coerced values are number text, percent text, time text, date-time
text, TRUE/FALSE, and a blank or text-number cell. The engine matches every cell, and numbers.md "Near-zero
snapping" records the rule for coerced addends as VERIFIED.

## Context
- `snap_near_zero(x, y, total)` (crates/rustsheet-engine/src/arithmetic.rs) is applied at the formula root by
  `Model::evaluate_formula_root` to the two addends of the final `+`/`-` after coercion. Every probe behind the rule
  uses number operands: literals, number cells, the powers-of-two `Inputs` of snap-addsub, arrays (T-3047) and
  pickers (T-3049). No cell has a text, boolean or blank addend, so applying the rule to coerced values is an
  inference.
- Coercion has its own rounding paths, and they could interact with the snap:
  - percent text is the decimal shifted and rounded once (coercion.md "Percent text is rounded once", so
    `"23.123%"` is not 23.123/100);
  - date-time text builds the full serial and keeps the subtraction error (dates.md "TIMEVALUE of \"date time\" text
    keeps serial-subtraction error": `TIMEVALUE("7/7/2026 4:35 PM")` = 0.69097222221898846);
  - time-only text is exact;
  - number text is cut to 15 digits.
  So `="10:00"-10/24`, `="7/7/2026 4:35 PM"-46210-(16+35/60)/24`, `="30%"-0.1-0.2` and `="0.3"-0.1-0.2` leave
  residues of a few ULP that a number-only snap would remove. If Excel coerces at a later stage, or snaps
  before coercion, they stay. Blank cells and booleans are exact (0, 1), but `=TRUE-(1-3*2^-52)` against
  `=1-(1-3*2^-52)` tests whether a boolean addend takes part in the bound.
- Why opus: this is numeric. Each form must be swept over k ULP (k = 0..9) around both sides' boundaries, with
  exact doubles built in formulas, because typed literals are cut. Where the coerced value is itself inexact
  (date-time text, percent), the residue's ULP count has to be computed from the oracle's own coerced value
  (`VALUE(text)` in a companion cell), not assumed. The answer may also interact with the operand-order question of
  T-3100 (left operand versus smaller addend). Keep the coerced operand on the left in one half of the cells and
  on the right in the other, and record which bound fits once T-3100's evidence is in.
- Method: the `excel-probe` skill (excel-graph). One case `probes/t3140-snap-coerced-operands`, milestone tag M2.
  Put a `VALUE(..)` or `N(..)` companion cell next to each formula so the coerced addend's exact value is in the
  oracle sidecar.

## Acceptance criteria
- [ ] `probes/t3140-snap-coerced-operands` is committed with its excel-graph sidecar. It holds at least 120 formula
      cells, covering each operand kind above: number text, percent text, time text, date-time text, TRUE/FALSE,
      a blank cell and a text-number cell. Each kind is used as a literal and from a cell, on the left and the
      right, at k values on both sides of the 8-ULP bound, with companion cells for the coerced values.
- [ ] numbers.md "Near-zero snapping" has a paragraph "Coerced addends" with the rule and cells (VERIFIED). If the
      snap sees the coerced value, the engine needs no change and the paragraph says so. If not, the paragraph
      says at which stage Excel snaps.
- [ ] Every cell of the case passes. Any change stays in `snap_near_zero` and `evaluate_formula_root` (no second snap
      helper). A unit test in crates/rustsheet-engine/src/test/test_near_zero_snapping.rs covers one coerced form
      per kind that the probe shows to differ from the number-only rule. snap-addsub (probes-numeric),
      t3047-snap-array-operands, t3049-snap-under-pickers and the gen-m2 cells show 0 regressions
      (`make eval-check`). `make eval-baseline` is run.
- [ ] Any cell that only the open operand-order rule explains is named in T-3100, so `make exit-owners MILESTONE=M2`
      reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The operand-order bound itself (T-3100) and LAMBDA or name bodies (T-3105).
- Snap inside SUM/AVERAGE over coerced range values. Range text is skipped by SUM, and direct text arguments are a
  separate question: file a follow-up if this probe suggests SUM's snap differs.
- Typed cell input of these texts (the cell stores a number then; no coercion at evaluation).
