---
id: T-3196
title: "Harness volatile scan looks up each formula word in every defined name: calipers pv26_010 (44,028 names) spends 3 min 48 s before its 2.7 s run"
milestone: M11
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The eval harness's volatile-cell scan of an xlsx case costs time roughly linear in its formulas and names. The set of
cells it skips as volatile stays exactly the same.

## Context
Gap triage 2026-10-30. On origin/main c59497fc, `rustsheet-eval show calipers/pv_goldens/pv26_010/golden.xlsx` takes
3 min 48 s of wall time. The case's own run (`millis` in report.json) is 2,664 ms; the rest goes to the harness before
the engine starts. Callgrind (same command; killed at 255G instructions) puts 99% of it in
`rustsheet_eval::runner::prepare` → `volatile::scorable_cells` → `volatile::volatile_cells`. pv26_010 has 53,315
formula cells and 44,028 defined names. pv26_2_010, pv23_057 and pv23_059 (43,054 or 44,028 names) are the same
shape, and pv23_042 (1,586 names, 120,586 formulas) spends 7.6G of 39.5G instructions there. Every `make eval` run with
calipers fetched pays this, case by case.

Code: crates/rustsheet-eval/src/volatile.rs.
- `with_names` (T-0045) runs once per formula cell. For every identifier-like word it calls `find`, which walks the
  whole `names` slice twice (sheet scope, then workbook scope) with `eq_ignore_ascii_case`. That is about
  cells × words × names comparisons.
- The taint loop in `volatile_cells` filters the whole `ranges` vector for every tainted cell (`by_range`). That is
  quadratic too, but only when there are volatile cells and many range precedents.

Fix: build a map from (ASCII-lowercased name, scope) to body once per `volatile_cells` call, and keep the sheet-scope
first, then workbook-scope order and the `seen` dedup exactly. If the profile still shows `by_range` once that is
done, index the ranges by sheet (and row band). Search for an existing name-map helper in the harness before writing
one (CLAUDE.md directive 3).

## Acceptance criteria
- [ ] `with_names` does no linear scan of `names` per word. The existing volatile.rs tests pass unchanged, and a new
      test covers a sheet-scoped name shadowing a workbook name with different case.
- [ ] `rustsheet-eval show calipers/pv_goldens/pv26_010/golden.xlsx` finishes in under 15 s of wall time on a release
      build (from 3 min 48 s); the Outcome gives before/after for pv26_010 and pv23_042.
- [ ] `make eval` (and with calipers, corpus-bench and poi fetched) reports the same skipped-volatile counts per case
      as before (`skipped_volatile` in report.json) and no baseline change; `make check` green; reviewers approve.

## Out of scope
- Engine load time with many names (T-3195) and the engine's SUMIFS time on pv23_042 (T-0498).
