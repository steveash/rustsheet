---
id: T-1041
title: "Oracle sweep: normal PDF (NORM.S.DIST/NORM.DIST/LOGNORM.DIST density, PHI)"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's normal density is pinned down on a dense grid, so T-0975 can fit its last digits against oracle values.

## Context
Split from T-0966 (one sweep per kernel family), which explains why: Excel's distribution values are not the correctly rounded ones (`docs/excel-semantics/statistics.md`), and blind fits against the few cached cells in `ironcalc-calc` regressed other cells (T-0925, decision 0026). Follow the `excel-probe` skill: hand-written spec cases (`rustsheet-case/v1`, `"expected": {}`) under `eval/suites/probes-numeric/cases/` (or generated with `rustsheet-gen --policy numeric-rules`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Split a large grid over several case files (one per function or argument block) so one rejected workbook cannot hide the rest; never put an arity-invalid call in a probe (T-0040 found that it makes Excel for the web reject the whole workbook). Dependent fitting task(s): T-0975.

## Acceptance criteria
- [x] Probe cases cover: `NORM.S.DIST(z,FALSE)` and `PHI(z)` for z in -38..8 step 0.125; `NORM.DIST(x,m,s,FALSE)` with s in {0.5, 1.2, 3, 10} on a coarser x grid; `LOGNORM.DIST(x,m,s,FALSE)` for a few (m, s) pairs.
- [x] The case files are committed with their `*.oracle.json`, and no cell is left pending (`make eval` shows none pending for them).
- [x] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no RUSTSHEET_ORACLE_* values in logs or commits.
- [x] New quirks visible in the sweep (values that differ from the correctly rounded result in a pattern, error edges, argument coercion) are recorded in `docs/excel-semantics/statistics.md`, citing the case ids.
- [x] If the oracle is unavailable (`rustsheet-oracle status` does not show `excel-graph OK`), set `status: blocked` with that command as the unblock check.
- [x] T-0975 links the committed case ids in its Context.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing any implementation (the dependent fitting tasks do that).

## Outcome
Seven probe case files (`m9-normpdf-*`) with oracle sidecars, 1,346 cells, none pending; findings in `docs/excel-semantics/statistics.md` (PHI == NORM.S.DIST pdf; sqrt(2*pi) hypothesis refuted on 305/369; subnormal flush to 0). No eval baseline change (probes only).
