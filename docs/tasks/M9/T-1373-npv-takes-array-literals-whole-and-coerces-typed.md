---
id: T-1373
title: "NPV takes array literals whole and coerces typed text and booleans; FVSCHEDULE is #VALUE! for text"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
NPV and FVSCHEDULE treat their non-scalar arguments as Excel does.

## Context
SPEC-0011 (docs/specs/SPEC-0011-scalar-argument-lifting.md) makes NPV's values and FVSCHEDULE's schedule `WholeByCaller` parameters: the lifting driver passes them whole, and this task's rules decide what is inside them. NPV's rate lifting is T-1372's.

probes/fin-argument-modes rows H368, H370, H371 and H382-H384 (excel-graph, 2026-10-11, T-1084; financial.md "Financial
function argument modes"):
- `NPV(0.1,{-10000,2750,4250,3250,2750})` is 302.233453999042 in Excel, but the engine gives 0.
- `NPV(0.1,-10000,"2750",TRUE,4250)` coerces the typed "2750" and TRUE (Excel -3914.6233180793665); the engine skips them.
- `NPV(0.1,-10000,"x",4250)` is `#VALUE!` in Excel; the engine skips the text.
- FVSCHEDULE is `#VALUE!` for a text cell in the schedule range and for text in a schedule literal; the engine skips the text.
NPV's ranges already skip text and booleans correctly (H369). Check whether the aggregate argument walker the
statistical functions use (`aggregate` mode) already does this, and reuse it.

## Acceptance criteria
- [x] H368, H370, H371, H382, H383 and H384 of probes/fin-argument-modes pass. The ulp miss on H368's value, if any, is T-0901's.
- [x] `make check` green; reviewers approve.

## Out of scope
- The ulp misses of NPV's value (T-0901).

## Outcome
NPV now takes array literals whole, coerces typed text and booleans and returns #VALUE! for typed text that is no number; FVSCHEDULE is #VALUE! for text in its schedule. Findings in financial.md "NPV and FVSCHEDULE arguments". H371 and H382-H384 now pass; H368 and H370 differ only by ulp (T-0901's). Eval delta: probes fixed 10 cells (fin-argument-modes H371, H382-H384; fin-fvschedule A3, A4, A13, A14, A16, A19), no regressions; the new probe npv-fvschedule-typed (16 cells) covers array literals, names, functions, parentheses and typed FVSCHEDULE scalars.

## Handoff
Branch `agent/functions-financial/T-1373-npv-arrays` (draft PR) has the behavior the oracle evidence supports: array literals read whole (text/bool/empty skipped), typed args coerced, `#VALUE!` for non-numeric typed text, single-cell/parenthesised/`@` references skipped (NPV.xlsx K5/L5), FVSCHEDULE `#VALUE!` for text/booleans in ranges and literals, `"0.1"` literal = one rate. Probe `probes/npv-fvschedule-typed` (A1-A16, excel-graph) is filled; remaining misses there are NPV ulp (T-0901). `make check` was green before the last review.

Reviewer excel-fidelity requested changes twice. Open finding (round 2, MAJOR): `fn_npv` decides "typed vs reference" by node shape (`ReferenceKind | ImplicitIntersection | OpRangeKind` after `.ungrouped()`), so references written other ways coerce text/booleans where `main` skipped them: `IF(TRUE,B1)`, `INDEX(B1:B2,1)`, a name `=Sheet1!$B$1`, `Sheet1:Sheet1!B1` coerce; `CHOOSE(1,B1)` and `OFFSET(B1,0,0)` skip; SUM treats all of them as references. No oracle covers them.
Next: decide "is a reference" with the existing aggregate machinery (`pick_reference`/`RefArg` in functions/util.rs ~315-380), or on the evaluated result, rather than node shape; add probes for `IF(TRUE,B1)`, `INDEX(B1:B2,1)`, a name pointing at a cell, `CHOOSE(1,B1)`, `OFFSET(B1,0,0)`, a 3-D ref (B1="2750"), plus `NPV(0.1,{1,#N/A})` and `NPV(0.1,SEQUENCE(3))`; narrow financial.md "Typed means anything but a reference" to what is probed. Run the reviewers again.
