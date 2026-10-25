---
id: T-0073
title: "Progress report: evidence depth section and top-40 most-used functions table"
milestone: M0
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-0070]
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
- Tier, attributed cells and share: the report's T-0068 fields, as rendered by T-0070.

## Acceptance criteria
- [ ] Module `progress_report/evidence.py` writes the `evidence` block:
  - in-scope functions per Excel-cell band (< 20, 20–199, 200–399, 400–999, ≥ 1000);
  - the median;
  - how many are `covered` (decision 0094);
  - a histogram SVG built with `progress_report/svg.py`.
- [ ] It writes the `top-functions` block: the 40 in-scope functions with the highest `prior`,
      ties by name, with category, tier, attributed cells, within-15-digit share and Excel
      cells.
- [ ] tools/tests/test_progress_report.py covers the bands at their edges (19, 20, 399, 400),
      the median of an even count, and the tie order.
- [ ] The regenerated page is committed. `make py-check` green; `make check` green; reviewers
      approve.

## Out of scope
- Usage weighting from real workbooks (T-0079).
- Argument coverage (T-0077).
