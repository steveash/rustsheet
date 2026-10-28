---
id: T-0089
title: "Milestone exit rows for M1, M2 and M7 to M11 in the eval report, per the new exit-measure decision"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0088]
---

## Goal
The eval report has a `milestone_exit` row for every Phase 1 milestone (M1, M2, M5 to M11), measured as decision
0085 and T-0088's decision define, so `make eval` says which Phase 1 milestone exits are met and
`scripts/exit_owners.py --milestone M<k>` lists the failing exit cells of any of them.

## Context
- `crates/rustsheet-eval/src/milestone_exit.rs` covers M5 and M6 only (T-2177). T-0088 writes the decision for the
  rest; this task implements it, reusing the existing row inputs (catalog coverage, generated suites, function
  scope, probe milestones, milestone lists) without scoring anything new.
- The progress page (tools/src/rustsheet_tools/progress_report/) shows the rows in its Milestones table; check it
  renders the new rows.
- Decision 0146 (T-0088) defines the measures; each new row reads at visible parity, `n/a` (never met) on a missing
  input, null verdict on a measure shown but not required:
  - **M1:** gen-m1 ≥ 3,000 scored cells, 0 visibly off; smoke 0 visibly off; the `m1-probes` list (not a probe row)
    0 visibly off. Needs a visibly-off count and cell list on each `milestone_lists` row.
  - **M8:** gen-m8a and gen-m8b each ≥ 3,000 scored cells, 0 visibly off (the row reads two generated suites;
    `exit_owners.py` reads only `gen-<milestone>` today); the poi subset (formulas with a defined name or a
    structured, 3-D or external reference, classified by the engine's parser) ≥ 99.5% not visibly off, deviations
    out, a crashed or errored subset case failing it as `case!*`; the M8 probe row 0 visibly off. Needs the poi
    subset count in the report. The classifier walks the parsed formula of an untrusted workbook: it returns a result for
    every formula, never panics and recurses no deeper than the parser's own bound (CLAUDE.md directive 5).
  - **M9:** the cumulative M9 function-scope row less exceptions with ≥ 0.9995 not visibly off (allowance
    `floor(0.0005 * scored)`); every in-scope M9 function `covered` and `meets_rate` in `v15_checkpoint` (no second
    count); the M9 probe row 0 visibly off. Exit-owners also lists the `attributed_off` cells of each M9 function
    below the rate.
  - **M10:** gen-m10 ≥ 10,000 displayed-text items with 0 failing and its value cells 0 visibly off; the M10 probe
    row 0 visibly-off value cells and 0 failing text items. Needs text-item counts and failing text cells on probe
    rows; exit-owners lists failing text items as `case!Sheet!A1`.
  - **M11:** corpus-bench ≥ 200,000 scored Excel-backed formula cells with ≥ 0.998 not visibly off (`n/a` when not
    fetched); the M11 probe row 0 visibly off.
  - Coverage and ironcalc-calc are `n/a` for M1, M8, M10 and M11; M9 shows coverage without requiring it.
- The M1 and M8-M11 criteria were T-2889, T-2890, T-2891 and T-0487's follow-ups; those tasks are dropped as
  superseded, so this task carries their rows and exit-owners support.

## Acceptance criteria
- [ ] `milestone_exit::MILESTONES` (or its replacement) covers M1, M2 and M5 to M11 with decision 0146's measures;
      unit tests cover a met and an unmet row for each new kind of measure.
- [ ] `make eval` report.md and report.json show the rows; `python3 scripts/exit_owners.py --milestone M<k>` runs for
      M1 and M8 to M11 and lists the cells decision 0146 names per criterion.
- [ ] `make progress-report` renders them; `make check` green; reviewers approve.

## Out of scope
- Making the milestones pass (their tasks).
