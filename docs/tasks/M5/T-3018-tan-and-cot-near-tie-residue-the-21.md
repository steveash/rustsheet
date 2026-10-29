---
id: T-3018
title: "TAN and COT near-tie residue: the 21 TAN and 19 COT probe cells the double-rounded tangent misses"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
Find which part of Excel's tangent decides the last bit at the 21 TAN near-ties that `excel_tan` misses. Then either
ship a model that matches more of them without losing a matched cell, or record in numbers.md why no deterministic
model can.

## Context
Gap triage 2026-10-28 (`make eval` on dea28aa5). No open task names these cells. T-2147 (done) shipped
RN53(RN64(tan)) and left them as "known failures of that probe". T-2664 (done) probed COT and found that it
inherits the same misses. Every cell is 1 or 2 ulp off and none is visibly off, so the task is deferred (decision 0114).
- probes-numeric/t2147-tan-double-rounding.case.json (21): Calc!B8, B22, B25, B49, B58, B59, B66, B73, B81, B106, B107,
  B111, B115, B123, B128, B130, B132, B135, B140, B145, B149
- probes-numeric/t2664-cot-double-rounding.case.json (19): Calc!B8, B22, B49, B58, B59, B66, B73, B81, B106, B107, B111,
  B115, B128, B130, B132, B135, B140, B145, B149
- Examples (excel-graph, 2026-10-25/26): `TAN(-26.3054)` (B8) is -2.3775582498320422 in Excel and ...427 in ours;
  `TAN(-1333.5)` (B49) is -9.3913318701533 in Excel and -9.391331870153302 in ours; `COT(-709541)` (B106) is
  3.6532528709594536 in Excel and ...545 in ours.
- A lead from this triage, not tested yet: the misses grow with |x|. None of the 38 arguments with |x| <= 1.5 misses.
  The 21 misses are at |x| = 15.5, 26.3, then 1333 to 1.08E+8, which is 19 of the 90 arguments above 1000. T-2664 saw the
  same in COT, which matches all 38 cells with |x| <= 1.5. T-2147 tried guard-bit variants of the final rounding
  (65..72 bits) and none did better beyond noise. So the reduction, not the rounding, may decide these bits. The
  remainder `x - n*pi/2` as Excel forms it (the pi64 model of `reduce` in
  crates/rustsheet-engine/src/functions/math_and_trigonometry/trig.rs) can be exact enough for SIN and COS, which
  round once, and still too coarse for a tangent that sits near a 64-bit tie. Candidates: the remainder rounded or
  truncated to 64 bits before the kernel (as FPREM1 leaves it in a register), and a 66-bit pi as some x87 reductions
  use. T-2147 already rejected the quotient of 64-bit sine and cosine (63/150).
- T-2147 filled the TAN probe twice with identical results, so these bits do not change between fills, unlike LN's
  (decision 0112). That makes a deterministic model plausible.
- numbers.md "TAN misses this model by 1 ulp" has the counts so far (129/150 TAN, 131/150 COT).
- Fit with the fit lab (`cd tools && uv run rustsheet-fitlab`). Decision 0056 applies: two attempts, then time-box.

## Acceptance criteria
- [ ] Either `excel_tan` changes and the two probe cases above match more cells than today (TAN 129/150, COT 131/150),
      with no TAN, COT, SIN or COS cell that passes today failing (probes-numeric m9-trig-reduction and
      trig-odd-even-sumxmy2-sweep, probes/trig-huge-argument, eval/baseline/gen-m5.txt and gen-m6.txt: no new
      known-failure lines),
- [ ] or numbers.md records each reduction and rounding model tried, with its TAN and COT match counts, and says why
      the cells stay named known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SIN and COS (they match the pi64 model on every probed cell).
- IMTAN and IMCOT (T-2774, T-2787).
