---
id: T-1042
title: "Oracle sweep: GAMMA and GAMMALN"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's GAMMA and GAMMALN are pinned down across their domain, so T-0973 can fit them against oracle values.

## Context
Split from T-0966 (one sweep per kernel family), which explains why: Excel's distribution values are not the correctly rounded ones (`docs/excel-semantics/statistics.md`), and blind fits against the few cached cells in `ironcalc-calc` regressed other cells (T-0925, decision 0026). Follow the `excel-probe` skill: hand-written spec cases (`rustsheet-case/v1`, `"expected": {}`) under `eval/suites/probes-numeric/cases/` (or generated with `rustsheet-gen --policy numeric-rules`), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`. Split a large grid over several case files (one per function or argument block) so one rejected workbook cannot hide the rest; never put an arity-invalid call in a probe (T-0040 found that it makes Excel for the web reject the whole workbook). Dependent fitting task(s): T-0973.

## Acceptance criteria
- [ ] Probe cases cover: `GAMMA(x)` and `GAMMALN(x)` at integers 1..171; half-integers -10.5..50.5 (GAMMALN too, recording whatever error Excel returns where it is undefined); x in (0,1) step 0.05 and near 0 (1e-300, 1e-10, 1e-5); negatives between poles (e.g. -0.5, -1.5, -2.25, -9.9); error edges: 0, negative integers, 171.7, GAMMALN of a negative.
- [ ] The case files are committed with their `*.oracle.json`, and no cell is left pending (`make eval` shows none pending for them).
- [ ] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] New quirks visible in the sweep (values that differ from the correctly rounded result in a pattern, error edges, argument coercion) are recorded in `docs/excel-semantics/statistics.md`, citing the case ids.
- [ ] If the oracle is unavailable (`rustsheet-oracle status` does not show `excel-graph OK`), set `status: blocked` with that command as the unblock check.
- [ ] T-0973 links the committed case ids in its Context.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing any implementation (the dependent fitting tasks do that).
