---
id: T-3785
title: "Gate A slice 4o: reference tables for LOGNORM.DIST, LOGNORM.INV, EXPON.DIST, WEIBULL.DIST, FISHER and FISHERINV"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3783]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
LOGNORM.DIST, LOGNORM.INV, EXPON.DIST, WEIBULL.DIST, FISHER and FISHERINV each get a committed, oracle-filled `s1`
Gate A table with a cited mpmath definition, and `make check` runs them. Every failing point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029 "Gate A holds"): on main (fafb6c6f) these six M9 statistical functions are
  missing tables, and no open task or pending branch names them. They are the closed-form continuous distributions
  and transforms around LN and EXP.
- It depends on T-3783 because LOGNORM.DIST and LOGNORM.INV reuse its normal cdf and quantile definitions
  (`defs/normal.py`): one definition per concept (CLAUDE.md directive 3).
- The math, at the double of each written literal: the lognormal as the normal of ln x; the exponential and Weibull
  cdf and density in closed form (1 - e^-t through expm1 at working precision); FISHER as atanh and FISHERINV as tanh.
- Semantics the domain and sampler must read (statistics.md):
  - "LOGNORM.INV is EXP of NORM.INV's quantile (T-0925)", and "One incomplete-gamma kernel ... (T-1778)":
    `LOGNORM.DIST(x,m,s,TRUE)` = `NORM.DIST(LN(x),m,s,TRUE)`.
  - "FISHER, FISHERINV, EXPON.DIST and HARMEAN are Excel operators around LN and EXP (T-2437)": FISHER near 0 is
    Excel's cancellation, not atanh. `FISHER(1.49172367472348E-11)` is 5 digits of 15 off atanh, and
    `FISHER(1.02759470998055E-16)` is 0. That is a relative error past decision 0188 R3's 1e-4 against the math, so
    those points are R3 candidates. List them for an owner ruling, or leave the region out of the table and say so
    in the definition. Either way, do not tune the definition to Excel's bits.
  - "WEIBULL.DIST sweep (T-1046)", "WEIBULL.DIST's cdf switches from expm1 to 1-EXP(-t) near t = 1 (T-2239)" and
    "WEIBULL's density divides by l^k: #DIV/0! when it underflows, #NUM! when it overflows (T-2239)": these give the
    error domain. The density's `#DIV/0!` where its limit is 0 is an Excel error that the table matches; it is not a
    math value.
  - "Normal densities are #NUM! once the square overflows (T-2435)" applies to LOGNORM.DIST's density.
- Owners for failing points: T-0925 (LOGNORM.INV central), T-0968 (LOGNORM.DIST cdf residue, blocked), T-0975
  (LOGNORM.DIST density), T-0978 (WEIBULL.DIST last digits), or a new task.
- Tools: as in T-3549 and T-3586. The definitions go in a new `defs/continuous.py`; FISHER and FISHERINV can live there
  or in their own module.
- Why opus: FISHER's evidenced cancellation is an R3 or out-of-table call, and three error domains (Weibull's l^k,
  the overflowing square, LN of x at 0) have to be read from the evidence.

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against Microsoft's published examples or the semantics entries' values. The lognormal definitions call
      `defs/normal.py` and do not copy it.
- [ ] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each of the six functions, generated, filled
      (excel-graph) and committed. `rustsheet-eval accuracy` reports `missing tables` lower by 6, with 0 failing points
      and 0 table defects.
- [ ] Every point outside the envelope is a `known-failures.tsv` line with an open owner task. The FISHER near-zero
      choice is stated, and any R3 candidate is listed in the Outcome for an owner ruling and not marked in this PR.
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The legacy names LOGNORMDIST, LOGINV, EXPONDIST and WEIBULL (T-3786 rules on legacy names).
- HARMEAN and the other sample-array statistics (T-3786).
- Changing any engine kernel.
