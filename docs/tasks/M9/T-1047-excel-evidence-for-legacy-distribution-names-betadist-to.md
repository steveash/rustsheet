---
id: T-1047
title: "Excel evidence for legacy distribution names BETADIST to FTEST"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The legacy compatibility names BETADIST, BETAINV, BINOMDIST, CHIDIST, CHIINV, CHITEST, CONFIDENCE, CRITBINOM, EXPONDIST, FDIST, FINV and FTEST each have Excel evidence (decision 0031), so `report.md` "Excel evidence per function" no longer lists them as unevidenced.

## Context
Split from T-0966 (one sweep per kernel family), which explains why: Excel's distribution values are not the correctly rounded ones (`docs/excel-semantics/statistics.md`), and blind fits against the few cached cells in `ironcalc-calc` regressed other cells (T-0925, decision 0026). Follow the `excel-probe` skill: hand-written spec cases (`rustsheet-case/v1`, `"expected": {}`) under `eval/suites/probes-numeric/cases/` (or generated with `rustsheet-gen --policy numeric-rules`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Split a large grid over several case files (one per function or argument block) so one rejected workbook cannot hide the rest; never put an arity-invalid call in a probe (T-0040 found that it makes Excel for the web reject the whole workbook). No fitting task depends on this; it closes an evidence gap (decision 0031).

## Acceptance criteria
- [ ] Probe cases cover: one `excel-probe` case per function with >= 25 cells: typical values, the same arguments as the modern equivalent (to show whether the alias is exact), argument coercion (text numbers, booleans, empty cells) and error edges (out-of-domain values, non-integer counts).
- [ ] The case files are committed with their `*.oracle.json`, and no cell is left pending (`make eval` shows none pending for them).
- [ ] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] New quirks visible in the sweep (values that differ from the correctly rounded result in a pattern, error edges, argument coercion) are recorded in `docs/excel-semantics/statistics.md`, citing the case ids.
- [ ] If the oracle is unavailable (`rustsheet-oracle status` does not show `excel-graph OK`), set `status: blocked` with that command as the unblock check.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing any implementation (the dependent fitting tasks do that).

## Outcome
Added 12 probes-numeric cases `m9-legacy-*` (one per function, 33-92 cells) with excel-graph sidecars, none pending. All aliases equal their modern equivalents; quirks recorded in `docs/excel-semantics/statistics.md`.
