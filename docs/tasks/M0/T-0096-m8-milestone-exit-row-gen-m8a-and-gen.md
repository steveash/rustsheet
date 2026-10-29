---
id: T-0096
title: "M8 milestone exit row: gen-m8a and gen-m8b, the poi subset at 99.5%, M8 probes (decision 0146 item 2)"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-0089, T-0094]
---

## Goal
The eval report has an M8 `milestone_exit` row measured as decision 0146 item 2 defines, and
`scripts/exit_owners.py --milestone M8` lists every cell and blocked case that keeps M8 from exiting.

## Context
- Decision 0146 (T-0088, PR #2144) item 2, required measures:
  - (a) gen-m8a and gen-m8b each ≥ 3,000 scored cells with 0 visibly off (3,577 and 3,368 cells, both 100%, on
    2026-10-28);
  - (b) the poi M8 subset (T-0094's report section) ≥ 99.5% not visibly off, deviations out of the denominator;
    a crashed or errored case holding a subset formula fails it and is listed as `case!*`; `n/a` (never met) when
    poi was not run or the section is absent;
  - (c) the M8 probe row 0 visibly off (9 on 2026-10-28, owned by T-2763, T-2855 and T-2911).
  - Coverage and ironcalc-calc are `n/a` (M8 has no functions in the catalog).
- Builds on T-0089's list of required suites per row (two generated suites here) and reads T-0094's counts; it
  scores nothing itself.
- `scripts/exit_owners.py` today reads one `gen-<milestone>` suite; the `case!*` form for a blocked case must be
  owned by a task naming the case stem, as decision 0139 reads a case. `scripts/` is protected: amend decision
  0146's Consequences (or add a record) in the same PR.
- T-2892 (gate gen-m8a and gen-m8b) is independent: this row reads the suites whether they are gated or not.

## Acceptance criteria
- [ ] report.md and report.json have an M8 row with the three measures; unit tests cover a met row, each measure
      failing alone (a visibly-off gen-m8b cell, a subset below 99.5%, a blocked subset case, a visibly-off probe
      cell) and the `n/a` row without poi.
- [ ] `python3 scripts/exit_owners.py --milestone M8` runs and lists cells from (a), (b), including `case!*`
      entries, and (c); `scripts/test_exit_owners.py` covers each.
- [ ] The Outcome note records the row's numbers with poi fetched and the unowned count
      `make exit-owners MILESTONE=M8` prints.
- [ ] `make progress-report` renders the row; `make check` green; reviewers approve.

## Out of scope
- Classifying poi cells (T-0094); gating gen-m8a/gen-m8b (T-2892).
- Owning or fixing the M8 cells it lists.
