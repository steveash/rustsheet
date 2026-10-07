---
id: T-1040
title: "Oracle sweep: normal CDF and erfc (NORM.S.DIST cumulative, ERFC, ERFC.PRECISE, GAUSS)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's normal CDF and erfc are pinned down on a dense grid, so T-0968 can fit Excel's formulation against oracle values instead of the few cached upstream cells.

## Context
Split from T-0966 (one sweep per kernel family), which explains why: Excel's distribution values are not the correctly rounded ones (`docs/excel-semantics/statistics.md`), and blind fits against the few cached cells in `ironcalc-calc` regressed other cells (T-0925, decision 0026). Follow the `excel-probe` skill: hand-written spec cases (`rustsheet-case/v1`, `"expected": {}`) under `eval/suites/probes-numeric/cases/` (or generated with `rustsheet-gen --policy numeric-rules`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Split a large grid over several case files (one per function or argument block) so one rejected workbook cannot hide the rest; never put an arity-invalid call in a probe (T-0040 found that it makes Excel for the web reject the whole workbook). Dependent fitting task(s): T-0968.

## Acceptance criteria
- [ ] Probe cases cover: `NORM.S.DIST(z,TRUE)` for z in -38..8 step 0.125; `ERFC(x)` and `ERFC.PRECISE(x)` for x in -6..27 step 0.125; `GAUSS(z)` on a coarser z grid (step 0.5); about 20 `NORM.DIST(x,m,s,TRUE)` and `LOGNORM.DIST(x,m,s,TRUE)` with s != 1.
- [ ] The case files are committed with their `*.oracle.json`, and no cell is left pending (`make eval` shows none pending for them).
- [ ] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] New quirks visible in the sweep (values that differ from the correctly rounded result in a pattern, error edges, argument coercion) are recorded in `docs/excel-semantics/statistics.md`, citing the case ids.
- [ ] If the oracle is unavailable (`rustsheet-oracle status` does not show `excel-graph OK`), set `status: blocked` with that command as the unblock check.
- [ ] T-0968 links the committed case ids in its Context.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing any implementation (the dependent fitting tasks do that).
