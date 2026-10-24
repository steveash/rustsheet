---
id: T-2328
title: "Fit XIRR's XNPV stop bound on long spans (T-1591 misses at root 0.1, T=20/40/120)"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A stop-bound form for XIRR's XNPV test that reproduces every cell of probes/xirr-span-zero-first (and the earlier fit set), replacing T-1583's span-scaled `span_stop`.

## Context
T-1591 (financial.md "XIRR is a bracketing bisection") found the span factor max(1, years) misses probes/xirr-span-zero-first A11, A14, A20 (root 0.1 at T = 20, 40, 120 years: Excel runs one lattice point past the model) while 80 years stops early; the bound is not monotone in the span. Lab: `cd tools && uv run rustsheet-fitlab xirr-oos`; model in tools/src/rustsheet_tools/fitlab/kernels/solvers.py (`span_stop`).

## Acceptance criteria
- [x] A candidate in the lab bit-exact on all numeric cells of `xirr-oos` and the XIRR fit set, with the rule recorded in financial.md.
- [x] `make check` green; reviewers approve.

## Out of scope
Engine changes (T-0916).

## Outcome
The stop bound was not the problem. The T-1591 misses (xirr-span-zero-first A11, A14, A20) come from the bracket.
Their root 0.1 is exactly the first chain point, so the sign of XNPV(0.1) picks the side, and Excel's XNPV gives
that sign by discounting with its own `^` (square-and-multiply, or exp(y·ln x)) and cutting literals to 15 digits.
New probe probes/xirr-exact-root-sign (excel-graph, 574 cells): Excel's sign matches its XIRR side on all 7
cells, and the lab XNPV (`solvers.xnpv` with `caret_pow`) fits all 541 numeric cells. The lab sweep parser now
cuts number literals to 15 digits (`sweep.number_literal`); `caret_pow` moved from yield_caret into solvers.
Candidate "bisect, first-value sign, span-scaled |f| bound, Excel ^ (T-2328)": 227/227 on `xirr` and 51/52 on
`xirr-oos`, where the one miss is A7, the input check. So T-1583's span-scaled bound stands. Recorded in
financial.md ("XNPV discounts with Excel's `^`" and the T-2328 bullet under "XIRR is a bracketing bisection").
Eval: 377 of the probe's 574 cells are new known failures in the engine (XNPV uses `powf`), follow-up T-2566.
