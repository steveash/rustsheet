---
id: T-0095
title: "M9 milestone exit row: ironcalc-calc at 0.9995, every M9 function at the Phase 1 rate, M9 probes (decision 0146 item 3)"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-0089]
---

## Goal
The eval report has an M9 `milestone_exit` row measured as decision 0146 item 3 defines, and
`scripts/exit_owners.py --milestone M9` lists the cells that keep M9 from exiting, so the planner and the steward
can give every one of them an owner (decision 0096).

## Context
- Decision 0146 (T-0088, PR #2144) item 3, required measures:
  - (a) the cumulative M9 function-scope row of ironcalc-calc less its exit exceptions
    (`eval/milestones/exit-exceptions.tsv`, decision 0087), with ≥ 0.9995 of its scored cells not visibly off:
    allowance `floor(0.0005 * scored)` (36 on 2026-10-28 against 138 visibly off);
  - (b) every in-scope function of catalog milestone M9 `covered` and `meets_rate` in `v15_checkpoint`
    (`crates/rustsheet-eval/src/v15_checkpoint.rs`, decisions 0094, 0118, 0143): read those rows, never a second
    count (directive 3); 181 of 263 met on 2026-10-28;
  - (c) the M9 probe row 0 visibly off (578 on 2026-10-28).
  - Catalog coverage is shown, not required; gen-m9 alone is not a criterion; "100% within 4 ULP" is Phase 1b and
    is not read.
- Exit-owners must list (a)'s visibly-off cells, the `attributed_off` cells (report.json, decision 0142) of each M9
  function failing (b), and (c)'s cells. `make unowned-off ALL=1` already lists `attributed_off` per function; reuse
  its reader in `scripts/exit_owners.py` rather than adding a second one.
- The rate constant: `milestone_exit::meets_rate` takes a numerator over `RATE_DEN = 1000`; 0.9995 needs a finer
  denominator or a second constant. Keep `meets_rate` the one definition of "at the rate".
- Builds on T-0089's per-milestone list of required suites. `scripts/` is protected: amend decision 0146's
  Consequences (or add a record) in the same PR.

## Acceptance criteria
- [x] report.md and report.json have an M9 row with the three measures, their counts beside the bit-level counts,
      and the count of M9 functions below the rate; unit tests cover a met row and each measure failing alone,
      and an exception line that keeps (a) met.
- [x] `python3 scripts/exit_owners.py --milestone M9` runs on a fresh `make eval` and lists cells from all three
      sources; `scripts/test_exit_owners.py` covers each source.
- [x] The Outcome note records the row's numbers and the unowned count `make exit-owners MILESTONE=M9` prints.
- [x] `make progress-report` renders the row; `make check` green; reviewers approve.

## Out of scope
- Owning or fixing the M9 cells the new report lists (planner and gap-triager follow-ups).
- Changing `v15_checkpoint`'s rate or attribution rule (decisions 0114, 0118).

## Outcome
- `milestone_exit` has an M9 row (decision 0146 item 3). It requires no suite (`gen_suite` empty, printed
  `none required`). (a) ironcalc-calc: the cumulative M9 function-scope row less its deviations and exit
  exceptions (`calc_scored`), met when at most `floor(0.0005 * calc_scored)` (`calc_allowance`) cells are visibly
  off, through `meets_rate` with `M9_CALC_RATE_NUM` = 9,995: every rate now has the denominator 10,000 (same
  shares; `v15_checkpoint.rate_num/rate_den` in report.json read 9990/10000). (b) `v15_checkpoint`'s rows carry
  each function's catalog `milestone`; the row counts the M9 ones (`rate_functions`) not `meets_rate`
  (`below_rate`, names in `below_rate_functions`). (c) the M9 probe row. Coverage is shown, not required. The
  report table has a new "functions at the Phase 1 rate" column (`n/a` on every other row).
- `scripts/exit_owners.py --milestone M9` lists the ironcalc-calc cells, the `attributed_off` cells of each
  function below the rate (`rate:<FUNCTION>`, through the reader `--all-off` uses) and the probe cells. The
  progress page shows the M9 measures (`goals.exit_measures`).
- `make eval` on this branch (calipers, corpus-bench and poi not fetched): M9 not met. ironcalc-calc 119 visibly
  off of 73,155 scored against an allowance of 36 [546 bit failures]; 74 of the 263 M9 functions below the
  rate; M9 probes 629 visibly off [2,613]; coverage 263/263. Exit cells: 119 ironcalc-calc, 99,994 `rate:`
  cells (BETA.DIST 51,668 and F.DIST 38,036 of them), 629 probe cells. `make exit-owners MILESTONE=M9`: 514
  unowned (459 `rate:` cells, 55 probe cells; every ironcalc-calc cell has an owner).
- Eval delta: none (report-only change).
