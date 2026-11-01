---
id: T-3784
title: "Gate A slice 4n: reference tables for ERF, ERF.PRECISE, ERFC and ERFC.PRECISE"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
ERF (one and two arguments), ERF.PRECISE, ERFC and ERFC.PRECISE each get a committed, oracle-filled `s1` Gate A table
with a cited mpmath definition, and `make check` runs them. Every failing point has an owner, and the two-argument
ERF's cancellation region gets an explicit treatment.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029 "Gate A holds"): on main (fafb6c6f) these four M9 engineering functions are
  missing tables. No open task and no pending branch names them for a table. Decision 0228 covered M5 to M7 only.
- The math: mpmath `erf`/`erfc` at the double of each written literal. ERF(a, b) is erf(b) - erf(a) at working
  precision.
- Semantics the domain and sampler must read (statistics.md):
  - "Normal CDF and ERFC sweep (T-1040)": ERFC and ERFC.PRECISE are bit-identical on the grid and 0 from
    x = 26.625 on, where the true value is subnormal. The zero tail is part of the domain: decide from that entry and
    numbers.md "Subnormal results become 0" whether those points are drawn with Excel's 0 as an evidenced flush, or
    left out.
  - "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)" and the T-1916
    entries after it: ERF is odd, ERF(d) = fl(1 - ERFC(d)), ERFC(-d) = fl(2 - ERFC(d)), and tiny x keeps the
    leading term (`ERF.PRECISE(2^-k)*2^k` = 1.1283791670955128 for k from 27 to 79).
  - The same entries: two-argument `ERF(a,b)` is the exact difference of the two rounded single values, a multiple
    of 2^-53. Where a and b are close, that difference can be far off erf(b) - erf(a) relative to the result, maybe
    past decision 0188 R3's 1e-4. Decide in this task how the sampler treats that region. It can draw it and list
    the points as R3 candidates for an owner ruling (a new R3 case needs a decision record with one, CLAUDE.md
    directive 1), or it can leave the region to the eval suites, which already hold Excel's cells for it. Record
    the choice in the definition's docstring.
  - engineering.md: `ERF(5.9)` is 0.9999999999999999 and `ERF(6)` 1; numeric text such as `ERF("0.5")` is accepted.
    A table drives numeric literals only, so the text form stays in the eval suites.
- Owners for failing points: T-0968 (cdf and erfc residue, blocked), or a new task. The probes `t1916-erf-wrappers`
  and `t1916-erfc-exp-ties` show the engine's ERF kernel bit-exact on their sweeps, so few failures are expected.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, `rustsheet-accuracy math --suite probes-numeric`). The definitions go in a
  new `defs/erf.py`.
- Why opus: the two-argument cancellation region is an R3 or out-of-table design call, and the subnormal tail needs a
  ruling against the evidence.

## Acceptance criteria
- [ ] `tools/src/rustsheet_tools/accuracy/defs/erf.py` has one definition per function, each citing its sources and
      stating the ERF(a, b) cancellation choice, with unit tests against the semantics entries' values above.
- [ ] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for ERF, ERF.PRECISE, ERFC and ERFC.PRECISE,
      generated, filled (excel-graph) and committed. ERF's table has both one- and two-argument points.
      `rustsheet-eval accuracy` reports `missing tables` lower by 4, with 0 failing points and 0 table defects.
- [ ] Every point outside the envelope is a `known-failures.tsv` line with an open owner task. Any R3 candidate is
      listed in the Outcome for an owner ruling and not marked in this PR.
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- NORM.S.DIST and GAUSS (T-3783), CHISQ.DIST and GAMMA.DIST (T-3786 classifies them).
- Changing the engine's erf kernel.
