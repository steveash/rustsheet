---
id: T-3287
title: "XIRR's negative-root stop over spans of several years: does the XNPV bound scale with the span? (follow-up of T-3268)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Settle the two questions T-3268 left open in XIRR's stop rule, both backed by oracle cells:
(1) does the XNPV bound on a negative root scale with the span, and (2) is the switch the midpoint's sign or the bracket's
side? Correct the wording in docs/excel-semantics/financial.md that the T-3268 reviews flagged.

## Context
- T-3268 (#2441) applies the XNPV bound |XNPV(m)| < 2e-8·max|value| to negative midpoints only, with no span factor.
  On every negative-root cell where the bound holds the stop back (A4 family, A23, the fit set's downward chain), the
  latest date is within a year of the first.
- The m9-fin-irr-rate cells with negative roots over longer spans (Calc!A603-A609, 2.43 years; A639-A645, 5.00 years)
  stop at the first step-test pass at |XNPV| <= 1.67e-8·max|value|. They pass whether or not the bound is scaled, so they
  do not discriminate (excel-fidelity review of #2441).
- T-3353 answers question (1) for two-flow series: probes/xirr-two-flow-long-span has negative roots -0.9 to -0.01
  over 3 to 100 years where the scaled bound misses 49 cells and the unscaled bound (the engine) matches all 270
  (financial.md, T-3353 bullet). Three-flow and negative-guess cells and question (2) remain here.
- Doc fixes (reviews of #2441):
  - financial.md's T-3268 bullet says "every negative-root cell has a latest date within a year of the first" and
    "(none is probed)". Narrow both to the cells where the bound binds.
  - Mark the older T-1591 and T-2328 bullets' "the span factor stays" as superseded by T-3268.
- Probe design (excel-probe skill): two- and three-flow negative roots (-0.02 to -0.9) over spans of 2, 5, 20 and 80 years,
  chosen so that |XNPV| at the first step-test pass lies between 2e-8 and 2e-8·years times max|value| (screen candidates
  with tools/src/rustsheet_tools/fitlab/kernels/xirr.py traces first). Add negative-guess cells, which walk only the
  downward chain, and positive roots reached on the downward chain if any exist, to separate the midpoint's sign from the
  bracket's side.
- Engine: `compute_xirr` in crates/rustsheet-engine/src/functions/financial/financial_util.rs.

## Acceptance criteria
- [ ] A probe case with oracle values exists for multi-year negative roots where the scaled and unscaled bounds disagree.
- [ ] The lab candidate that fits the probe is named in kernels/xirr.py. The engine follows it, or the task records that the
      T-3268 rule already fits. No XIRR cell that passes today may regress.
- [ ] financial.md's T-3268 bullet is corrected and the older span bullets are marked superseded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR's input checks (xirr-early-dates A7) and error orders (T-2611).
