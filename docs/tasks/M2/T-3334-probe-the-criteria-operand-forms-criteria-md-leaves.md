---
id: T-3334
title: "Probe the criteria operand forms criteria.md leaves UNVERIFIED after T-3136: <, > and <= over formatted number text, month names with a time, $1e2, a percent at the 15-digit tie, CHAR(160)-padded operands"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence settles each criteria-operand form that docs/excel-semantics/criteria.md still
marks UNVERIFIED in its "VALUE's grammar" paragraph. Each one ends `VERIFIED` with its probe cells,
or the engine gap is owned by a filed follow-up. The forms are: the `<`, `>` and `<=` operators
over formatted number operands, VALUE forms beyond T-3136's set, a percent operand at the
15-digit tie, and CHAR(160)-padded formatted text. The *IF(S) criteria grammar is M2 scope.

## Context
- criteria.md, the paragraph after "Verified in text.md (T-3178)" (about line 510), lists as UNVERIFIED:
  "other VALUE forms (month names with a time, `1e2` with `$`), the `<`, `>` and `<=` operators (by
  analogy with `>=`), a percent rounding at the 15-digit tie, CHAR(160)-padded formatted text".
  Non-en-US locales are out of the project's scope (en-US is the spec).
- T-3136 (probes/t3136-criteria-formatted-number-operands) probed 14 operands under the plain,
  `=`, `<>` and `>=` operators only. It found that `>=` with a numeric operand never matches a
  text cell. The engine applies the same to `<`, `>` and `<=` by analogy (`criterion_text_number`
  and the operand reader in crates/rustsheet-engine/src/functions/criteria.rs). That analogy is
  unprobed.
- Why opus: the percent tie needs operands designed around 15-digit rounding, so the probe
  separates three readings: the operand cut to 15 digits before the division by 100, after it, or
  never. Examples: `"33.3333333333333%"` and `"33.33333333333335%"` against `=1/3` and
  `=0.333333333333333`, and the `<`/`>` neighbours of each. criteria.md "Numeric `<`, `<=`, `>`,
  `>=` criteria compare the 15-digit value of the cell" gives the cell side. The operand side at a
  tie is the open question. Any finding also needs judging against SUMIFS's indexed path.
- Shapes to cover, each from a literal and from a cell, under SUMIF, SUMIFS, COUNTIF, COUNTIFS
  and DSUM as T-3136 did, with power-of-two SUMIF weights so each sum names the matched cells:
  - operators `<`, `>`, `<=` with T-3136's operands (`50%`, `$1`, `1,000`, `(1)`, `12:00 PM`, `1e2%`)
    over the same number and text cells;
  - `"Jan 1 2020 12:00"`, `"1-Jan 6:00 PM"`, `"January 1, 2020 0:00"`, `"$1e2"`, `"$1E+2"`, `"1e2$"`;
  - the percent ties above;
  - `"$1"&CHAR(160)`, `CHAR(160)&"50%"`, `"1,000"&CHAR(160)` as operands, and as text cells under a
    plain numeric criterion;
  - `VALUE` of every operand beside it (column H, as in T-3136), so a row that disagrees with VALUE
    stands out.

## Acceptance criteria
- [ ] A new probe case `probes/t3334-criteria-operand-forms` (excel-probe skill, tag `M2`), filled
      with excel-graph and committed with its sidecar. It covers every bullet above.
- [ ] criteria.md's UNVERIFIED sentence is replaced by a rule with the probe's cells as evidence,
      or narrowed to what is still unprobed and why. Any quirk found is written there in this PR.
- [ ] Every cell of the new case that the engine fails is owned by a follow-up functions-criteria
      task filed in this PR, listing the cells, before `make eval-baseline` records them as new
      coverage. `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine fix (the follow-up task).
- Bare `"<="`/`">="` against empty text, `$(1)`-style number text and USDOLLAR (T-3119).
- Locales other than en-US.
