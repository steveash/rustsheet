---
id: T-1046
title: "Oracle sweep: WEIBULL.DIST"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's WEIBULL.DIST is pinned down on a small grid, so T-0978 can fit its last digits against oracle values.

## Context
Split from T-0966 (one sweep per kernel family), which explains why: Excel's distribution values are not the correctly rounded ones (`docs/excel-semantics/statistics.md`), and blind fits against the few cached cells in `ironcalc-calc` regressed other cells (T-0925, decision 0026). Follow the `excel-probe` skill: hand-written spec cases (`rustsheet-case/v1`, `"expected": {}`) under `eval/suites/probes-numeric/cases/` (or generated with `rustsheet-gen --policy numeric-rules`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Split a large grid over several case files (one per function or argument block) so one rejected workbook cannot hide the rest; never put an arity-invalid call in a probe (T-0040 found that it makes Excel for the web reject the whole workbook). Dependent fitting task(s): T-0978.

## Acceptance criteria
- [ ] Probe cases cover: `WEIBULL.DIST(x,k,l,TRUE/FALSE)` for k in {0.5, 1, 2, 3.7}, l in {1, 9} on an x grid including small x (1e-10) and the far tail.
- [ ] The case files are committed with their `*.oracle.json`, and no cell is left pending (`make eval` shows none pending for them).
- [ ] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] New quirks visible in the sweep (values that differ from the correctly rounded result in a pattern, error edges, argument coercion) are recorded in `docs/excel-semantics/statistics.md`, citing the case ids.
- [ ] If the oracle is unavailable (`rustsheet-oracle status` does not show `excel-graph OK`), set `status: blocked` with that command as the unblock check.
- [ ] T-0978 links the committed case ids in its Context.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing any implementation (the dependent fitting tasks do that).
