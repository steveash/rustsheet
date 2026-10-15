---
id: T-1643
title: "Legacy CHOOSE operator arm under INDEX, MODE and LOOKUP: @ placement and values"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Legacy formulas with a CHOOSE operator arm under INDEX, MODE and LOOKUP give Excel's FORMULATEXT and values.

## Context
Found by T-1638: probes/legacy-choose-arm-aggregate-and-array-constant-at cells H1, H3 (`=INDEX(CHOOSE(1,B1:B6*1),3)`: Excel shows no `@` on the INDEX call and gives #REF!, ours `=@INDEX(...)` and #VALUE!), I10 (`=MODE(CHOOSE(1,B1:B6*1))` off-row: Excel #VALUE!, ours #N/A) and J1 (`=LOOKUP(2,CHOOSE(1,B1:B6*1))`: Excel 1, ours #N/A). docs/excel-semantics/references.md, "Legacy `@` on CHOOSE operator arms under aggregates". Code: crates/rustsheet-engine/src/expressions/parser/static_analysis/legacy.rs.

## Acceptance criteria
- [ ] H1, H3, I10 and J1 of that probe pass; baseline updated with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
H1, H3, I10 and J1 of probes/legacy-choose-arm-aggregate-and-array-constant-at now pass (baseline -4, no regressions). Three small changes: `static_analysis_index` returns Scalar for INDEX over a CHOOSE with arithmetic arms (no `@` on the call); `index_impl` reads a scalar picked by IF or CHOOSE as a one-element array (`#REF!`); MODE propagates the error of a computed (non-reference) argument; LOOKUP reads a scalar lookup_vector as a one-element array. Generalising the INDEX scalar rule fixed 12 more cells but unmasked function-name-forms A4: filed as T-1645.
