---
id: T-1684
title: "Probe PV, FV and NPER outside the fitted sweep: non-integer nper, pmt 0, pv or fv 0, rates near -1"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the PV, FV and NPER inputs that T-0901's fitted formulations cover only by
extrapolation, so financial.md's "Annuity evaluation order" holds on the whole input domain.

## Context
T-0901 implemented the fitted forms (financial.md "Annuity evaluation order"). The fitting sweeps
(probes-numeric/m9-fin-pv-fv, m9-fin-nper-npv) have integer nper and non-zero pmt only. Its fidelity
review flagged three extrapolations:
- PV/FV with a non-integer nper: `^` falls back to exp(y·ln x) there (numbers.md);
- NPER with pmt = 0: the old code used `-fv/pv`, the fitted form computes `(-fv*r)/(pv*r)`; also pv
  = 0 or fv = 0;
- rates within a few ulps of -1: `1+r` uses Excel's `+` (`extended_sum`), which snaps near-zero sums.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, ≥ 40 cells, oracle-filled) covers those three groups for PV,
  FV and NPER, including type 1.
- [ ] financial.md records the result: VERIFIED where the fitted form holds, a new rule otherwise.
  Any failing cell has a filed task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PMT, IPMT, PPMT and NPV (T-1528).
