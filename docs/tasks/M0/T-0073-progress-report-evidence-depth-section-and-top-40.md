---
id: T-0073
title: "Progress report: evidence depth section and top-40 most-used functions table"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-0081]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
The dashboard shows how much Excel evidence backs each function. It also shows how the 40 most-used
functions stand: these are the bulk of what everyday workbooks call.

## Context
- SPEC-0022, the `top-functions` and `evidence` rows of "Page layout".
- Evidence: report.json `v15_checkpoint.functions[]` (`cells` = Excel-evidence cells, `covered`;
  decision 0094's bars are 400 and 200) and the evidence floor of 20 cells.
- Usage: the catalog's `prior` (real-world frequency; eval/catalog/functions.yaml). It is not in
  scope.json, so read it with the existing loader `rustsheet_tools.gen.generator.Catalog.load`,
  as catalog_view.py does. Do not add a second YAML parse, and do not change scope.json:
  `CatalogFunction` in Rust denies unknown fields.
- Tier, attributed cells and share: the report's T-0068 fields, as rendered by T-0081.

## Acceptance criteria
- [x] Module `progress_report/evidence.py` writes the `evidence` block:
  - in-scope functions per Excel-cell band (< 20, 20–199, 200–399, 400–999, ≥ 1000);
  - the median;
  - how many are `covered` (decision 0094);
  - a histogram SVG built with `progress_report/svg.py`.
- [x] It writes the `top-functions` block: the 40 in-scope functions with the highest `prior`,
      ties by name, with category, tier, attributed cells, within-15-digit share and Excel
      cells.
- [x] tools/tests/test_progress_report.py covers the bands at their edges (19, 20, 399, 400),
      the median of an even count, and the tie order.
- [ ] The regenerated page is committed. `make py-check` green; `make check` green; reviewers
      approve.

## Out of scope
- Usage weighting from real workbooks (T-0079).
- Argument coverage (T-0077).

## Outcome
- `progress_report/evidence.py` writes the `top-functions` and `evidence` blocks (SPEC-0022
  sections 7 and 8) and `docs/progress/evidence.svg` (one-series `svg.bar_chart`). Shared-file
  edits are kept small: two `page.LAYOUT` entries and one `body` merge in page.py; a `--catalog`
  option and the evidence chart in `__init__.py`; `FunctionParity` gains `evidence`
  (`v15_checkpoint.functions[].cells`), `covered` and `prior`, filled by `compute`'s new optional
  `priors` argument.
- Priors are read with `Catalog.load` (`evidence.load_priors`); no second YAML parse, scope.json
  unchanged, no task files read. A prior that is not a non-negative number, or an in-scope
  function the catalog lacks, fails the run. A function with no prior (NOW, TODAY) sorts last.
- The floor (20) and decision 0094's bars (400, 200) are constants in evidence.py citing
  `EVIDENCE_FLOOR`, `CELLS_BAR` and `GENERATED_BAR`; they only set band edges and text.
  `covered` is the report's verdict.
- Regenerated from the T-0081 run (2026-10-25, `05703c6c`, 0 skipped suites); the rerun is
  byte-identical.
  - Evidence: median **722.5** Excel cells per in-scope function; < 20: 0, 20–199: 5,
    200–399: 4, 400–999: 331, ≥ 1000: 166; **492 of 506** covered (decision 0094). The 9 below
    400 are RAND, NOW, RANDARRAY, TODAY, RANDBETWEEN, INFO, GETPIVOTDATA, TRUE and FALSE. The
    assessment had a median of 715 and 9 below 400: its count is the same, and the median moved
    with later evidence or its one-off method.
  - Top 40 (SUM … CHOOSE, ties at prior 3, 2 and 1.5 broken by name): 22 never visibly off,
    26 meet the Phase 1 rate. Lowest within-15-digit shares: VLOOKUP 45.73%, MIN 94.00%,
    COUNT 95.76%, MAX 96.05%, COUNTA 96.25%, SUMPRODUCT 98.80%.
- Gates run here: `make py-check`, `make hygiene`, `scripts/check_governance.py origin/main`
  and `make typos`; `make check` and the reviewers are for the PR (last criterion open).
- Tests (tools/tests/test_progress_report.py): bands at 19, 20, 399 and 400, the median of six
  functions (209.5) and a whole median, the prior order with ties by name and the missing-prior
  last, the priors' validation, and the page and chart written by `main`.
