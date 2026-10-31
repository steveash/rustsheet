---
id: T-3291
title: "Oracle sweep: PERCENTILE and PERCENTILE.INC at a tiny k, where a + h(b - a) and (a - h a) + h b round apart"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
defer: phase-1b
---

## Goal
T-3292 can fit the last step of Excel's PERCENTILE.INC interpolation against a dense excel-graph sweep. Today it has
one cell where the documented rule misses and four where the form that fixes that cell misses.

## Context
- statistics.md "PERCENTILE and PERCENTILE.INC interpolate from a one-based rank 1 + k(n-1)" (T-1811): Excel takes
  r = 1 + k(n - 1), j = floor(r), h = r - j and returns a[j] + h(a[j+1] - a[j]). That rule matches 39 of 40 gen-m9
  cells. The miss is gen-m9/5395b8a427764a3f Calc!A269 `PERCENTILE(Data!A9:D11,1E-16)`. Its numbers are -2.5, 1E-15,
  0.1, 0.2, 100 and 123456789.123 (n = 6; Data!D9 holds the text "1" and Data!A10 the text "#N/A"). Excel gives
  -2.499999999999999 and the engine -2.4999999999999987.
- Checked in doubles (gap triage 2026-10-30): h = 4.440892098500626E-16. `a + h*(b - a)` and `a + (h*b - h*a)` give
  -2.4999999999999987. `(1 - h)*a + h*b` and `(a - h*a) + h*b` give Excel's -2.499999999999999, since -2.5·(1 - h) is
  a rounding tie that goes to even. The statistics.md entry says `(1-h)*a + h*b` misses 4 other cells. The candidates
  differ only where such ties or near-ties occur, and the cached cells hold one of them.
- This task only gathers evidence. The fit is T-3292's.

## Acceptance criteria
- [ ] A new probes-numeric case `eval/suites/probes-numeric/cases/t3291-percentile-tiny-k.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a description naming this task, as in
      `m9-fin-power.case.json`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite
      probes-numeric` (excel-probe skill). Its cells:
      - control: `=PERCENTILE({0.1,123456789.123,1E-15,100,-2.5,0.2},1E-16)` and the same call as
        `PERCENTILE.INC`. They show whether an array literal gives A269's -2.499999999999999.
      - grid: `=PERCENTILE.INC({a,b,P...},k)`, with P a padding of the value 1E+6 repeated so the array has n
        numbers. a ranges over {-2.5, -1, -0.3, 0.7, 3, -1000}, b over {1E-15, a + 0.5, a + 7} (skip any b that is not
        above a), n over {2, 3, 6, 11}, and k over {1E-16, 2E-16, 3E-16, 5E-16, 1E-15, 3E-15, 1E-14, 0.1, 0.3, 0.7}.
        Write the literals as the shortest decimal that round-trips: each cell's `hypotheses` label (or the
        description) records a, b, n and k.
      - twins: the first 40 grid cells repeated with `PERCENTILE` and with `QUARTILE.INC` where k is 0.25 or 0.75
        (add k = 0.25 and 0.75 rows for n = 6 and 11 if the grid has none).
      Keep every call at valid arity (an invalid one makes Excel for the web reject the whole workbook, T-0040). Split
      the case in two if the oracle's size limits need it.
- [ ] The `.oracle.json` sidecar is committed with no pending cells, and `make eval` loads the case with no crash.
- [ ] statistics.md's PERCENTILE entry gets one Evidence line naming the new case. No rule changes here.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting the form and any engine change (T-3292).
- PERCENTILE.EXC and QUARTILE.EXC, whose zero-based rank is another formula.
