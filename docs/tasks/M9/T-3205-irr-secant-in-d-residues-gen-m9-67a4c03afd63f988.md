---
id: T-3205
title: "IRR secant-in-d residues: gen-m9 67a4c03afd63f988 A395 (16 ulp), t2565 P!C157 and the fit set's last misses"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3120]
campaign: visible
---

## Goal
The bit-level residue T-3120's secant in d (financial.md "IRR is a secant in d = 1 - v") leaves: gen-m9
67a4c03afd63f988 Calc!A395 returns Excel's bits, or the task records why no candidate gets it without losing
fit-set cells.

## Context
- Follow-up of T-3120 (decision 0176). All three cells are visibly off at the 15th digit: A395 and A214 by
  2.2E-16 (half-unit 5E-17), P!C157 by 7.9E-11 relative. A395 is a regression T-3120 accepted.
- gen-m9/67a4c03afd63f988 Calc!A395 `IRR(Data!C10:E12,0.5)`, flows {0.5,0.1,-0.5}: Excel -0.09501243788791103,
  ours -0.0950124378879108 (16 ulp). Every variant starting from d0 = 1 - 1/(1 + g) gets it and scores 235/263
  on the fit set against the shipped 262/263; at guess 0.5 the fit set's `{-20,5,5,5,5,5}` and
  `{-1000,230,200,125,200,10}` cells prefer the shipped start.
- probes-numeric/m9-fin-irr-rate Calc!A214 `IRR({-1000,230,200,125,200,10},5)`: 16 ulp (the fit set's one miss).
- probes/t2565-irr-guess-sweeps P!C157 (flow 67a4c03a x-1, guess an iterate): about 100 grid steps off since
  T-2565; the secant in d does not move it.
- Code: `irr_secant`, `irr_npv`, `compute_irr` (crates/rustsheet-engine/src/functions/financial/financial_util.rs);
  fit lab `irr_secant_d` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py).
- From T-3120's review (minor, folded in here): (1) dropping leading zero flows rests on three cells, all with a root
  near 0.1; probe a few leading-zero flows with far roots (the 1e-7 residual scales by x^k). (2) The lab's
  `_npv_v_running` does not flush subnormals as the engine's `discounted_sum` does; flush or say so. (3) Add an
  "unconditional zero-step stop" candidate to `secant_d_candidates()` so financial.md's 6-cell claim is reproducible,
  and pin the zero-step bound with a probe if one can separate the bounds.
- Decision 0056: each attempt is a fit-lab run of named candidates; two attempts that do not raise the
  bit-exact count block the task as time-boxed.

## Acceptance criteria
- [x] Candidates are scored in the fit lab against the fit set plus A395 and A214; the Outcome records each.
- [x] Any shipped change keeps the fit set at 262/263 or better and removes A395 from eval/baseline/gen-m9.txt,
      with no new known-failure lines.
- [x] financial.md records the result, with the cells.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Visible stage (decision 0083): done when A395 and A214 are within half a unit of the 15th significant digit
of Excel's value with the fit set kept at 262/263 or better; P!C157 may stay with a sentence in financial.md.
Bit-level residue goes to a follow-up with `defer: phase-1b`.

## Out of scope
- IRR's fallback path digits (T-1686, T-2928) and `#DIV/0!` cells (T-2802).

## Outcome
Attempt 1 (decision 0056) shipped: IRR's NPV uses Excel's twice-rounded `*`, `/` and `+` (`discounted_sum`,
shared with worksheet NPV, whose final division is twice-rounded too). Both open cells, Calc!A214 and gen-m9 A395,
were exactly the secant's next iterate after the one the plain-double NPV let the stop tests accept.
Fit lab `irr` (263 cells, plus A395): shipped 263/263 and A395 (was 262/263); NPV in plain doubles (T-3120) 262;
unconditional zero-step stop 257 (now a lab candidate, reproducing financial.md's 6-cell claim); leading zeros kept
261; d0 = 1 - 1/(1+g) 236. Rejected arithmetic variants (64-bit NPV 194, right-to-left sum 197, c_i*v^i 179, three
step formulations 233-243, extra-step or two-small-step stops 47-50, step bound 1E-8 193) are listed in financial.md.
The lab's running-product NPV now flushes subnormals as the engine does (review item 2).
Eval: gen-m9 known failures 3181 -> 3178 (A395 of 67a4c03afd63f988, IRR A268 of 81bcdc0bec0c8e6b, NPV A85 of
9d9ceba257f8001b), 0 regressions. probes-numeric m9-fin-irr-rate Calc!A214 now matches. P!C157 stays (financial.md).
Not done: review item 1 (a probe of leading-zero flows with far roots) needs new oracle cells; it is recorded as
unprobed in financial.md.
