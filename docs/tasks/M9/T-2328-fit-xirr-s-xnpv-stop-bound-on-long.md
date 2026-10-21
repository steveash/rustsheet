---
id: T-2328
title: "Fit XIRR's XNPV stop bound on long spans (T-1591 misses at root 0.1, T=20/40/120)"
milestone: M9
lane: research
status: todo
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
- [ ] A candidate in the lab bit-exact on all numeric cells of `xirr-oos` and the XIRR fit set, with the rule recorded in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
Engine changes (T-0916).
