---
id: T-3336
title: "Oracle sweep: CHIINV and CHISQ.INV.RT at one degree of freedom in the right tail (p 1E-4 to 2^-54), with CHISQ.INV at 1-p and CHISQ.DIST at Excel's landing points"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The fit task T-3337 has dense excel-graph evidence of where Excel's CHIINV and CHISQ.INV.RT land at one degree of
freedom in the right tail, instead of the one visibly-off pair it has today.

## Context
- statistics.md "CHISQ.INV.RT and CHIINV quirks" (T-1634): at df 1 the right-tail quantile is the left one of 1 - p,
  and Excel's search runs on its double-valued lower cdf. Gap triage 2026-10-31 (`make eval` on origin/main,
  excel-graph; `scripts/exit_owners.py --all-off`, no open owner):
  - probes-numeric/m9-legacy-chiinv A1 `CHIINV(1e-09,1)` and A2 `CHISQ.INV.RT(1e-09,1)`: Excel 37.324892889999234,
    ours 37.324893106518715 (the complement crossing; the true quantile is 37.32489305136234, T-2561). Excel is about
    2.2E-7 below ours, one step of the cdf's resolution near 1 (an ulp of 1 - p is about 1.1E-16 and the density there
    about 5E-10), so it is visibly off (8th digit).
  - Yet probes-numeric/m9-chisq-inv-rt A1 `CHISQ.INV.RT(1e-10,1)` = 41.82145620298276 is 2 ulp from ours
    (41.82145620298279), and `CHISQ.INV.RT(0.0001,1)` = 15.1367052266236 1 ulp (m9-legacy-chiinv A12). So the
    landing step is not taken at every p: which p, and from which side, is unknown. Three points cannot fix it.
- Today's df-1 evidence: m9-legacy-chiinv A1, A2, A11, A12, A21, A22, A31, A32, A41, A42, A51, A52, A61, A62;
  m9-chisq-inv-rt A1-A7; t1049-chisqinvrt A1, A5, A9, A13, A17; t1543-inverse-cdf-tiny-p A37-A42; t3148-gamma-quantile-flat-band
  A64-A112 (CHISQ.INV.RT at p near 1). Nothing between 1E-4 and 1E-9 or between 1E-10 and 2^-54 (where 1 - p rounds
  to 1 and the call is `#NUM!`), and no CHISQ.DIST at Excel's landing points.

## Acceptance criteria
- [ ] A new probes-numeric case, `eval/suites/probes-numeric/cases/t3336-chisq-df1-right-tail.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in
      `m9-fin-power.case.json`), is filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill). Its grid:
      - `CHISQ.INV.RT(p,1)` and `CHIINV(p,1)` for p = m * 10^-k, m in {1, 1.5, 2, 3, 5, 7}, k from 4 to 16;
      - `CHISQ.INV(1-p,1)` for the same p, written as `1-<p>` so Excel rounds 1 - p itself;
      - `CHISQ.INV.RT(p,1)` for p = 2^-j, j from 40 to 54, and at 6E-17, 7E-17, 8E-17 and 9E-17 (the `#NUM!` edge between
        2^-54 and 1E-16 that statistics.md calls unprobed);
      - `CHISQ.DIST(x,1,TRUE)` and `1-CHISQ.DIST(x,1,TRUE)` at x = 37.324892889999234 and 37.324893106518715 and at the
        8 adjacent multiples of 2.2E-7 on each side of 37.3248930 (the step Excel took at 1E-9);
      - `CHISQ.INV.RT(p,1.5)` at p = 1E-9 and 1E-10 (df truncation).
      Keep each call at valid arity (an invalid one makes Excel for the web reject the whole workbook, T-0040).
- [ ] The `.oracle.json` sidecar is committed with no pending cells.
- [ ] docs/excel-semantics/statistics.md "CHISQ.INV.RT and CHIINV quirks" cites the case in its Evidence and records
      at which p Excel's value is one cdf step off the complement crossing (a table of p, Excel, ours, the step count).
- [ ] T-3337 links the case.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing (T-3337). df 2 and above (T-3311, T-3146, T-3310, T-3232).
