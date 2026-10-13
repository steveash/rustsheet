---
id: T-1043
title: "Oracle sweep: Student t (T.DIST, T.DIST.RT, T.DIST.2T, T.INV, T.INV.2T, CONFIDENCE.T)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's Student t functions are pinned down on a grid of degrees of freedom and tails, so T-0969 and T-0967 can fit them against oracle values.

## Context
Split from T-0966 (one sweep per kernel family), which explains why: Excel's distribution values are not the correctly rounded ones (`docs/excel-semantics/statistics.md`), and blind fits against the few cached cells in `ironcalc-calc` regressed other cells (T-0925, decision 0026). Follow the `excel-probe` skill: hand-written spec cases (`rustsheet-case/v1`, `"expected": {}`) under `eval/suites/probes-numeric/cases/` (or generated with `rustsheet-gen --policy numeric-rules`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Split a large grid over several case files (one per function or argument block) so one rejected workbook cannot hide the rest; never put an arity-invalid call in a probe (T-0040 found that it makes Excel for the web reject the whole workbook). Dependent fitting task(s): T-0969, T-0967.

## Acceptance criteria
- [ ] Probe cases cover: `T.DIST(x,df,TRUE)`, `T.DIST(x,df,FALSE)`, `T.DIST.RT(x,df)`, `T.DIST.2T(x,df)` for df in {1,2,3,4,5,10,30,100,1000,1e6} and an x grid that includes tails (x up to 1e6, where T-0969 must not compute 1 - cdf); `T.INV(p,df)` and `T.INV.2T(p,df)` on a p grid including values near 0, 0.5 and 1; a few `CONFIDENCE.T(alpha,sd,n)`; non-integer df (Excel truncates or not: the sweep shows which).
- [ ] The case files are committed with their `*.oracle.json`, and no cell is left pending (`make eval` shows none pending for them).
- [ ] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] New quirks visible in the sweep (values that differ from the correctly rounded result in a pattern, error edges, argument coercion) are recorded in `docs/excel-semantics/statistics.md`, citing the case ids.
- [ ] If the oracle is unavailable (`rustsheet-oracle status` does not show `excel-graph OK`), set `status: blocked` with that command as the unblock check.
- [ ] T-0969, T-0967 link the committed case ids in their Context.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing any implementation (the dependent fitting tasks do that).
