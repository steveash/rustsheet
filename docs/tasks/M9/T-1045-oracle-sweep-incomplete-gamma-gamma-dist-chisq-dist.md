---
id: T-1045
title: "Oracle sweep: incomplete gamma (GAMMA.DIST, CHISQ.DIST, CHISQ.DIST.RT, GAMMA.INV, CHISQ.INV, CHISQ.INV.RT)"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's incomplete-gamma family is pinned down on a grid of shape and scale, so T-0971 and T-0972 can fit it against oracle values.

## Context
Split from T-0966 (one sweep per kernel family), which explains why: Excel's distribution values are not the correctly rounded ones (`docs/excel-semantics/statistics.md`), and blind fits against the few cached cells in `ironcalc-calc` regressed other cells (T-0925, decision 0026). Follow the `excel-probe` skill: hand-written spec cases (`rustsheet-case/v1`, `"expected": {}`) under `eval/suites/probes-numeric/cases/` (or generated with `rustsheet-gen --policy numeric-rules`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Split a large grid over several case files (one per function or argument block) so one rejected workbook cannot hide the rest; never put an arity-invalid call in a probe (T-0040 found that it makes Excel for the web reject the whole workbook). Dependent fitting task(s): T-0971, T-0972.

## Acceptance criteria
- [ ] Probe cases cover: `GAMMA.DIST(x,a,b,TRUE/FALSE)` for a in {0.5, 1, 2, 9, 50, 500}, b in {1, 2, 1e6} on an x grid around each mean; `CHISQ.DIST(x,k,TRUE/FALSE)` and `CHISQ.DIST.RT` for a few k; `GAMMA.INV`, `CHISQ.INV`, `CHISQ.INV.RT` on a p grid including tails.
- [ ] The case files are committed with their `*.oracle.json`, and no cell is left pending (`make eval` shows none pending for them).
- [ ] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] New quirks visible in the sweep (values that differ from the correctly rounded result in a pattern, error edges, argument coercion) are recorded in `docs/excel-semantics/statistics.md`, citing the case ids.
- [ ] If the oracle is unavailable (`rustsheet-oracle status` does not show `excel-graph OK`), set `status: blocked` with that command as the unblock check.
- [ ] T-0971, T-0972 link the committed case ids in their Context.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing any implementation (the dependent fitting tasks do that).

## Outcome
Added 11 probes-numeric case files (6 GAMMA.DIST by a, CHISQ.DIST, CHISQ.DIST.RT, GAMMA.INV, CHISQ.INV, CHISQ.INV.RT; 467 cells) with excel-graph sidecars, none pending. Findings in `docs/excel-semantics/statistics.md`. T-0971 and T-0972 link the case ids.
