---
id: T-1643
title: "Legacy CHOOSE operator arm under INDEX, MODE and LOOKUP: @ placement and values"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: opus
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
H1, H3, I10 and J1 of probes/legacy-choose-arm-aggregate-and-array-constant-at now pass (baseline -4, no regressions). A legacy INDEX over a CHOOSE with arithmetic arms takes no `@` on the call (`legacy_call_reads_a_value`, legacy path only; INDEX's static result stays Unknown so dynamic formulas still spill); INDEX and LOOKUP's array form read a scalar picked by IF or CHOOSE as a one-element array (`scalar_as_array`); MODE propagates the error of an argument IF or CHOOSE picked. The IF half and the wider rules are inferred: T-1645 (INDEX of any scalar), T-1651 (MODE direct errors, LOOKUP scalar vectors).

## Handoff (resumed)
Resolved by the opus run: every review finding below was fixed (fidelity and quality re-reviews APPROVE).
Implemented on this branch (PR #861, now draft): H1, H3, I10, J1 pass and `make check` is green, but the excel-fidelity reviewer returned REQUEST_CHANGES (other reviewers approved) and I stopped at the session budget. What to fix, from that review (verify each against the code):
- BLOCKING: `static_analysis_index` (legacy.rs, wired into static_analysis.rs) returns Scalar for INDEX over a CHOOSE with arithmetic arms for *every* formula, and `set_cell_with_formula` (model.rs ~2479) uses it to decide whether a cell is dynamic. So dynamic `=INDEX(CHOOSE(1,B1:B6*1),0)` and `=INDEX(CHOOSE({1,2},B1:B6*1,B1:B6*2),0,2)` lose their spill (`#VALUE!`; main spills). Put the "no `@` on the INDEX call" rule only in the legacy `@`-placement path (where the CHOOSE-arm rules in legacy.rs live), keep the global INDEX analysis `Unknown`, and add a unit/eval case for the dynamic spill.
- MAJOR: MODE now propagates the error of any non-reference argument (`MODE(1/0,1,1)` and `MODE(NA(),1,1)` changed from skipping to an error). Only I10 (a CHOOSE pick) is evidenced. Narrow it to a CHOOSE/IF pick, or add probe cells (`MODE(1/0,1,1)`, `MODE(NA(),1,1)`, `MODE(#DIV/0!,"1")`, a name referring to an error cell, `MODE(A1)` with A1 an error) and mark the rest inferred.
- MAJOR: LOOKUP wraps *any* scalar lookup_vector as a one-element array (`LOOKUP(2,1)`, `LOOKUP("b","a")`, and the 3-argument form), but references.md:1256 records a scalar result_vector as `#N/A`. Narrow to the 2-argument form with a scalar picked by CHOOSE/IF (reuse `picks_a_branch`), or probe first.
- MAJOR (docs): references.md "Legacy `@` on CHOOSE operator arms" says VERIFIED but now states general rules; split the unprobed generalisations out as inferred, with a probe task, and mention the MODE/LOOKUP rules in coercion.md and lookup.md.
- The INDEX scalar wrap is limited to IF/CHOOSE picks on purpose (T-1645 has the generalisation and the function-name-forms A4 blocker); `scalar_as_array` in cast.rs is the one helper for LOOKUP and INDEX.
