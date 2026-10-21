---
id: T-2220
title: "Oracle sweep for the F distribution: F.DIST cdf and pdf and F.DIST.RT over a degrees-of-freedom grid (T-2221 fit)"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: sonnet
depends_on: []
---

## Goal
T-2221 can fit Excel's F-distribution formulation (closed forms, complement, tail choice and
density) against a dense excel-graph sweep. Today it has only about 130 scattered cells, in which
two PARTIAL rules contradict each other.

## Context
- statistics.md "Incomplete beta in the ironcalc-calc workbooks: F tails and densities (T-0970)" has
  two PARTIAL rules, "the complement is exact" and "both tails come from one evaluation", and
  oracle cells contradict both. t1049-fdistrt A2 and A8 fit the w-then-1-w form; m9-legacy-fdist
  A11 and A12 fit the exact complement. It also records a d1 = d2 = 1 arctangent lead that has only
  five points.
- Excel is not correctly rounded on this family. At (10,10,10), `F.DIST.RT(10,10,10)` =
  0.0005715525434020334 is 1.4E-15 above the true value (m9-f-dist-rt A16). At d1 = d2 = 1,
  `F.DIST(1,1,1,TRUE)` = 0.5000000000000001 and `F.DIST.RT(1,1,1)` = 0.4999999999999999 where the
  exact value is 0.5 (t1049-fdist A7, m9-legacy-fdist A31). Telling formulations apart needs many
  points at small df, where closed forms apply, and at moderate df.
- Existing web cases: probes-numeric/m9-f-dist (30 cells), m9-f-dist-rt (20), m9-legacy-fdist (90),
  t1049-fdist (43), t1049-fdistrt (27).

## Acceptance criteria
- [ ] A new probes-numeric case, `eval/suites/probes-numeric/cases/t2220-f-dist-sweep.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the
      inputs, as in `m9-fin-power.case.json`), is filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill). Its grid:
      - F.DIST(x, d1, d2, TRUE), F.DIST(x, d1, d2, FALSE) and F.DIST.RT(x, d1, d2) for d1, d2 in
        {1, 2, 3, 4, 5, 7, 10, 20, 50, 100} (all 100 pairs) and x in {0.1, 0.25, 0.5, 1, 1.5, 2, 3, 5, 10, 100}.
      - At d1 = d2 = 1 and d1 = 1 or d2 = 1 (closed-form candidates), also x at 30 points spread over
        1E-6 .. 1E6.
      - Keep each call at valid arity. An invalid one makes Excel for the web reject the whole workbook (T-0040).
        Keep the workbook under the oracle's size limits; split it into cases of at most about 1,500 cells if needed.
- [ ] The `.oracle.json` sidecars are committed with no pending cells.
- [ ] statistics.md's T-0970 section records what the sweep shows, citing the case: at which (d1, d2) Excel is
      correctly rounded, where it is not, and whether `F.DIST + F.DIST.RT` is exactly 1.
- [ ] T-2221 links the case.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing (T-2219, T-2221). F.INV and F.INV.RT (T-0974). df of 1E7 and above (T-1538).
