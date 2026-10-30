---
id: T-3205
title: "IRR secant-in-d residues: gen-m9 67a4c03afd63f988 A395 (16 ulp), t2565 P!C157 and the fit set's last misses"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3120]
defer: phase-1b
---

## Goal
The bit-level residue T-3120's secant in d (financial.md "IRR is a secant in d = 1 - v") leaves: gen-m9
67a4c03afd63f988 Calc!A395 returns Excel's bits, or the task records why no candidate gets it without losing
fit-set cells.

## Context
- Follow-up of T-3120 (decision 0176). All cells are within 15 significant digits except P!C157.
- gen-m9/67a4c03afd63f988 Calc!A395 `IRR(Data!C10:E12,0.5)`, flows {0.5,0.1,-0.5}: Excel -0.09501243788791103,
  ours -0.0950124378879108 (16 ulp). Every variant starting from d0 = 1 - 1/(1 + g) gets it and scores 235/263
  on the fit set against the shipped 262/263; at guess 0.5 the fit set's `{-20,5,5,5,5,5}` and
  `{-1000,230,200,125,200,10}` cells prefer the shipped start.
- probes-numeric/m9-fin-irr-rate Calc!A214 `IRR({-1000,230,200,125,200,10},5)`: 16 ulp (the fit set's one miss).
- probes/t2565-irr-guess-sweeps P!C157 (flow 67a4c03a x-1, guess an iterate): about 100 grid steps off since
  T-2565; the secant in d does not move it.
- Code: `irr_secant`, `irr_npv`, `compute_irr` (crates/rustsheet-engine/src/functions/financial/financial_util.rs);
  fit lab `irr_secant_d` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py).
- Decision 0056: each attempt is a fit-lab run of named candidates; two attempts that do not raise the
  bit-exact count block the task as time-boxed.

## Acceptance criteria
- [ ] Candidates are scored in the fit lab against the fit set plus A395 and A214; the Outcome records each.
- [ ] Any shipped change keeps the fit set at 262/263 or better and removes A395 from eval/baseline/gen-m9.txt,
      with no new known-failure lines.
- [ ] financial.md records the result, with the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IRR's fallback path digits (T-1686, T-2928) and `#DIV/0!` cells (T-2802).
