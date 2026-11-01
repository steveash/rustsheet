---
id: T-3784
title: "Gate A slice 4n: reference tables for ERF, ERF.PRECISE, ERFC and ERFC.PRECISE"
milestone: M0
lane: harness
status: done
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
- [x] `tools/src/rustsheet_tools/accuracy/defs/erf.py` has one definition per function, each citing its sources and
      stating the ERF(a, b) cancellation choice, with unit tests against the semantics entries' values above.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for ERF, ERF.PRECISE, ERFC and ERFC.PRECISE,
      generated, filled (excel-graph) and committed. ERF's table has both one- and two-argument points.
      `rustsheet-eval accuracy` reports `missing tables` lower by 4, with 0 failing points and 0 table defects.
- [x] Every point outside the envelope is a `known-failures.tsv` line with an open owner task. Any R3 candidate is
      listed in the Outcome for an owner ruling and not marked in this PR.
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- NORM.S.DIST and GAUSS (T-3783), CHISQ.DIST and GAMMA.DIST (T-3786 classifies them).
- Changing the engine's erf kernel.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/erf.py`: one definition each for ERF, ERF.PRECISE, ERFC and ERFC.PRECISE,
  with their sources. From |x| = 1E+4 erfc is the asymptotic series (A&S 7.1.23), because mpmath's own `erfc`
  overflows a float there.
- Model: a `Param` can now be `optional`, as a trailing parameter only. `Definition.params_for(n)` is the one place
  that maps a call's argument count to its parameters. The sampler, `compute`, `parse_formula` and the eval-cell
  math use it, so ERF's single table holds one- and two-argument points, and probes-numeric one-argument ERF cells
  now get math values.
- The ERF(a, b) cancellation region is left out of the table. A two-argument point is redrawn while
  |erf b - erf a| < 1e-9 * max(|erf a|, |erf b|), which keeps Excel's subtraction error near 2.2E-7 relative, far
  from R3's 1e-4. The docstring says why. The region's Excel cells stay in probes-numeric `t1916-erf-wrappers`.
- **R3 candidates, for an owner ruling (not marked; T-3794):** this PR commits the math sidecar of
  `t1916-erf-wrappers`. In it, 602 of the 915 two-argument cells are more than 1e-4 relative off the math, from
  3.9E-4 to 3.28. For example:
  - D1530 `ERF(0.5000000000000002,0.5000000000000003)`: Excel -2.220446049250313E-16, math 9.756446527736046E-17;
  - D1531: Excel 0, math 9.756E-17;
  - D1532: Excel 3.3306690738754696E-16.
  The engine matches Excel at all of them. The Gate A table does not depend on the ruling.
- The subnormal tail is drawn, not left out: Gate A's absolute floor 1e-300 covers Excel's flush to 0 from
  x = 26.625. The tail and edge strata reach it.
- Tables: `eval/accuracy/tables/{ERF,ERF.PRECISE,ERFC,ERFC.PRECISE}/s1`, 120 points each, filled from excel-graph
  (0 failures). `rustsheet-eval accuracy`: 4,316 points in 26 tables, 0 failing, 0 table defects, no new
  known failures. Missing tables went from 271 to 267.
- Sidecars: probes-numeric math sidecars were regenerated, and the seven ERF/ERFC cases gained a sidecar
  (t1040-erfc, t1040-erfc-precise, t1916-erf-small, t1916-erf-wrappers, t1916-erfc-exp-ties, t1916-erfc-mid,
  t1916-erfc-tail). The regeneration also produced sidecars for t3352, t3443 and t3568 cases that had none on main.
  Those cases are not part of this task, so they are not committed here.
- Semantics:
  - statistics.md, T-1778 two-argument bullet: the cancellation cells past 1e-4.
  - statistics.md, T-1040 entry: no argument size is an error for the four functions, up to |x| = 4.5E+298, the
    two-argument form included. Off the 0.125 grid, ERFC.PRECISE near DBL_MIN is up to 4E-14 off the math.
- Follow-up: T-3794 (owner ruling on R3 for the cancellation region, blocked on the owner).
- Eval delta: none in the gated suites (tools, tables and sidecars only).
