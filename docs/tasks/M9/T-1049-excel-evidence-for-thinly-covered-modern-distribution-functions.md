---
id: T-1049
title: "Excel evidence for thinly covered modern distribution functions (F.*, CHISQ.*, HYPGEOM.DIST, WEIBULL.DIST, Z.TEST, PHI, T.TEST)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
F.DIST, F.DIST.RT, F.INV, F.INV.RT, F.TEST, the CHISQ.* functions, HYPGEOM.DIST, WEIBULL.DIST, Z.TEST, PHI and T.TEST each have >= 25 Excel-evidenced cells (decision 0031).

## Context
Split from T-0966 (one sweep per kernel family), which explains why: Excel's distribution values are not the correctly rounded ones (`docs/excel-semantics/statistics.md`), and blind fits against the few cached cells in `ironcalc-calc` regressed other cells (T-0925, decision 0026). Follow the `excel-probe` skill: hand-written spec cases (`rustsheet-case/v1`, `"expected": {}`) under `eval/suites/probes-numeric/cases/` (or generated with `rustsheet-gen --policy numeric-rules`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Split a large grid over several case files (one per function or argument block) so one rejected workbook cannot hide the rest; never put an arity-invalid call in a probe (T-0040 found that it makes Excel for the web reject the whole workbook). It closes an evidence gap (decision 0031); T-0977 (HYPGEOM.DIST fit) waits on it.

## Acceptance criteria
- [ ] Probe cases cover: one `excel-probe` case per function with >= 25 cells, focused on what the kernel sweeps (T-1041, T-1044, T-1045, T-1046) do not cover: argument coercion (text numbers, booleans, empty cells, ranges), error edges (out-of-domain values, mismatched array sizes for F.TEST/T.TEST, T.TEST tails and type values), Z.TEST/T.TEST typical values, and a HYPGEOM.DIST grid of >= 60 cells (cumulative and not, small integer arguments, non-integer arguments (the sweep shows whether Excel truncates them), populations up to 1e6) that T-0977 can fit against.
- [ ] The case files are committed with their `*.oracle.json`, and no cell is left pending (`make eval` shows none pending for them).
- [ ] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] New quirks visible in the sweep (values that differ from the correctly rounded result in a pattern, error edges, argument coercion) are recorded in `docs/excel-semantics/statistics.md`, citing the case ids.
- [ ] If the oracle is unavailable (`rustsheet-oracle status` does not show `excel-graph OK`), set `status: blocked` with that command as the unblock check.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing any implementation (the dependent fitting tasks do that).
