---
id: T-1044
title: "Oracle sweep: incomplete beta (BETA.DIST, F.DIST, F.DIST.RT, BETA.INV, F.INV, F.INV.RT)"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's incomplete-beta family is pinned down on a grid of shape parameters, so T-0970 and T-0974 can fit it against oracle values.

## Context
Split from T-0966 (one sweep per kernel family), which explains why: Excel's distribution values are not the correctly rounded ones (`docs/excel-semantics/statistics.md`), and blind fits against the few cached cells in `ironcalc-calc` regressed other cells (T-0925, decision 0026). Follow the `excel-probe` skill: hand-written spec cases (`rustsheet-case/v1`, `"expected": {}`) under `eval/suites/probes-numeric/cases/` (or generated with `rustsheet-gen --policy numeric-rules`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Split a large grid over several case files (one per function or argument block) so one rejected workbook cannot hide the rest; never put an arity-invalid call in a probe (T-0040 found that it makes Excel for the web reject the whole workbook). Dependent fitting task(s): T-0970, T-0974.

## Acceptance criteria
- [ ] Probe cases cover: `BETA.DIST(x,a,b,TRUE/FALSE)` for a, b in {0.0001, 0.5, 1, 2, 2.5, 5, 10, 50, 100} on an x grid (choose a sparse x grid so the family stays a few hundred cells); `F.DIST(x,d1,d2,TRUE/FALSE)` and `F.DIST.RT` for a few (d1, d2); `BETA.INV`, `F.INV`, `F.INV.RT` on a p grid including tails.
- [ ] The case files are committed with their `*.oracle.json`, and no cell is left pending (`make eval` shows none pending for them).
- [ ] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] New quirks visible in the sweep (values that differ from the correctly rounded result in a pattern, error edges, argument coercion) are recorded in `docs/excel-semantics/statistics.md`, citing the case ids.
- [ ] If the oracle is unavailable (`rustsheet-oracle status` does not show `excel-graph OK`), set `status: blocked` with that command as the unblock check.
- [ ] T-0970, T-0974 link the committed case ids in their Context.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing any implementation (the dependent fitting tasks do that).

## Outcome
Added 14 probes-numeric case files (9 BETA.DIST by a, F.DIST, F.DIST.RT, BETA.INV, F.INV, F.INV.RT; 576 cells) with excel-graph sidecars, none pending. Findings in `docs/excel-semantics/statistics.md`. T-0970 and T-0974 link the case ids.
