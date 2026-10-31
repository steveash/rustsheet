---
id: T-3366
title: "POISSON.DIST and BINOM.DIST generalize poorly to fresh points (76% and 75% within 15 digits): find the cause and fix it"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
POISSON.DIST and BINOM.DIST are within 15 digits of Excel on fresh stratified points at about the rate they reach on the
public cases, with the cause of the gap found and fixed.

## Context
- Owner's 2026-10-31 assessment (decision 0188): on fresh stratified points scored against Excel for the web and 50-digit
  mpmath, POISSON.DIST is within 15 digits on 76% (96% on the public cases) and BINOM.DIST on 75% (90%).
- Code: statistical/poisson.rs (`fn_poisson_dist`, the incomplete gamma under incomplete_gamma/), statistical/binom.rs
  and discrete.rs (the exact walk and its statrs fallback, the Loader pmf).
- Related: T-3352 (P0, BINOM at a very large n), T-2146, T-3269, T-3295 (their own cells). Find the cause first with a
  fresh sweep (which strata fail, against Excel and against the math), then fix the kernel; never tune it to the sweep.

## Acceptance criteria
- [ ] A fresh stratified sweep (probes-numeric, filled; strata of decision 0188 R2) for each function is committed and the
      failing strata, with their cause, are recorded in statistics.md.
- [ ] After the fix the fresh within-15-digit share of each function is within a stated margin of its public share,
      shown on a second fresh sweep filled after the change (R4.2); any fitted piece follows R4.
- [ ] The Gate A envelope holds on both sweeps (decision 0188 R2).
- [ ] No gated cell regresses beyond decision 0083's allowance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity on cells already within 15 digits (Phase 1b).
