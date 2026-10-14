---
id: T-1048
title: "Excel evidence for legacy distribution names GAMMADIST to ZTEST"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The legacy compatibility names GAMMADIST, GAMMAINV, HYPGEOMDIST, LOGINV, LOGNORMDIST, NEGBINOMDIST, NORMDIST, NORMINV, NORMSDIST, NORMSINV, POISSON, TDIST, TINV, TTEST, WEIBULL and ZTEST each have Excel evidence (decision 0031), so `report.md` "Excel evidence per function" no longer lists them as unevidenced.

## Context
Split from T-0966 (one sweep per kernel family), which explains why: Excel's distribution values are not the correctly rounded ones (`docs/excel-semantics/statistics.md`), and blind fits against the few cached cells in `ironcalc-calc` regressed other cells (T-0925, decision 0026). Follow the `excel-probe` skill: hand-written spec cases (`rustsheet-case/v1`, `"expected": {}`) under `eval/suites/probes-numeric/cases/` (or generated with `rustsheet-gen --policy numeric-rules`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Split a large grid over several case files (one per function or argument block) so one rejected workbook cannot hide the rest; never put an arity-invalid call in a probe (T-0040 found that it makes Excel for the web reject the whole workbook). It closes an evidence gap (decision 0031); T-0977 (HYPGEOM.DIST fit) waits on its HYPGEOMDIST case.

## Acceptance criteria
- [x] Probe cases cover: one `excel-probe` case per function with >= 25 cells: typical values, the same arguments as the modern equivalent (to show whether the alias is exact), argument coercion (text numbers, booleans, empty cells) and error edges (out-of-domain values, TDIST tails 1/2, non-integer counts).
- [x] The case files are committed with their `*.oracle.json`, and no cell is left pending (`make eval` shows none pending for them).
- [x] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no RUSTSHEET_ORACLE_* values in logs or commits.
- [x] New quirks visible in the sweep (values that differ from the correctly rounded result in a pattern, error edges, argument coercion) are recorded in `docs/excel-semantics/statistics.md`, citing the case ids.
- [x] If the oracle is unavailable (`rustsheet-oracle status` does not show `excel-graph OK`), set `status: blocked` with that command as the unblock check.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing any implementation (the dependent fitting tasks do that).

## Outcome
16 probes-numeric cases `t1048-*` (34-102 cells each, 12 of the 16 functions paired with their modern equivalent), all filled by excel-graph with no pending cells. Result: every alias is exact except TDIST for negative x (#NUM! vs T.DIST.RT/2T). Quirks recorded in `docs/excel-semantics/statistics.md` ("Legacy distribution names sweep, second family"). No implementation change; eval baseline unchanged.
