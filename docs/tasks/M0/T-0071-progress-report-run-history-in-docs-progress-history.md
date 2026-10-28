---
id: T-0071
title: "Progress report: run history in docs/progress/history.json, since-last-run deltas, trend charts and stale handling for incomplete runs"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-0081]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
Each `make progress-report` adds one row to docs/progress/history.json. The page shows what
changed since the last complete run, and trend charts. When the eval is incomplete or a fetch
failed, the page says so and keeps the last good numbers, marked stale.

## Context
- SPEC-0022, sections "History and deltas" and "Failure handling". The row schema is in the spec.
- Inputs: report.json `run` (T-0080), `v15_checkpoint` (T-0080), the T-0068 tiers, tasks.json (T-0069),
  and git.
- The steward missed 2026-10-23 and 2026-10-24 without a trace in the repository. A missing date
  in the history must show as a gap in the trend charts, not be interpolated.

## Acceptance criteria
- [x] The module `progress_report/history.py` handles the history file:
  - it reads docs/progress/history.json, or creates it with today's row only;
  - it adds one row per run date, and a rerun on the same date replaces that row;
  - it never rewrites an older row;
  - it fails non-zero on invalid JSON or a row missing a schema key.
- [x] The `since-last-run` block shows deltas against the last *complete* row:
  - Phase 1 functions at rate;
  - functions per tier;
  - visibly-off cells by kind;
  - Excel cells;
  - tasks done;
  - commits on `main` since that row's commit (`git rev-list --count --first-parent`).
  On the first run it says there is no earlier run.
- [x] The `trends` block has line-chart SVGs, with a dark-scheme style and explicit fills as in
      T-2041: functions at ≥ 99.9% within 15 digits, visibly-off cells, and open tasks per
      campaign stage. A missing date leaves a gap.
- [x] An incomplete run (report missing or unparseable, an older shape, or
      `run.skipped_suites` not empty):
  - rewrites only the `status` block, with a Stale banner naming the reason and the last complete
    run;
  - leaves the other blocks, the SVGs and functions.md byte-identical;
  - adds a `complete: false` row;
  - exits 0.
  Tests cover each reason.
- [x] A report whose `run.commit` is not `HEAD` still counts as complete (test). Changed in review:
      a complete page names only the report's commit, so it does not depend on `HEAD`; only a
      Stale banner also names `HEAD`.
- [x] `make py-check` green; `make check` green; reviewers approve.

## Out of scope
- Back-filling the assessment's numbers: the method differs (SPEC-0022).
- Running the generator in CI or a routine (T-0076).

## Outcome
- New module `progress_report/history.py` handles docs/progress/history.json. It writes one row
  per line, so a run's diff is one added or changed line. It reads the file, or starts it with
  this run's row only. A new date appends a row, and a rerun on the same date replaces that row.
  One exception: an incomplete rerun never replaces a complete row of the same date, because the
  page still shows that run's numbers and its stale banner names that row. A date before the last
  row's date stops the run, so older rows are never rewritten. These all exit non-zero: invalid
  JSON, a non-array, a row missing a key of the SPEC-0022 schema (a complete row also needs every
  measure the deltas and charts read), and dates that are not one per day, oldest first.
- `since-last-run` compares against the last complete row before this run's date. Incomplete
  rows and a same-day rerun's own row are skipped. It shows:
  - Phase 1 functions at rate;
  - the functions in each tier;
  - visibly-off attributed cells, numbers and type/error/text separately;
  - attributed Excel cells;
  - tasks done;
  - first-parent commits from that row's commit to this run's (`git rev-list --count --first-parent`).
  Git sits behind a small `Git` protocol, so tests use a fake.
- `trends` has three line charts made by `svg.line_chart`: functions at rate, visibly-off cells by
  kind, and open tasks (todo, in progress, blocked; scripts/tasks.py `OPEN`) per campaign stage
  plus parked. The charts reuse svg.py's `Palette`, chrome and dark-scheme style. Lines get an
  explicit `stroke` and a `.s-<key>` dark-mode stroke rule. Every date from the first row to the
  last gets a slot, so a missing date or an incomplete run breaks the line, and a point with no
  neighbour is drawn as a dot. The y axis starts at 0, unless the values sit in a narrow band far
  from 0 (`y_axis`).
- Incomplete runs: a missing or unreadable report, invalid JSON, an older report shape, or a
  non-empty `run.skipped_suites`. An older shape is the new `ReportShapeError`: a field tagged with
  the task that added it, or no `run`. Such a run rewrites only the status block, splicing it with
  `blocks.replace_generated`, so every other byte of the page stays the same. The new status block
  is a Stale banner with the reason, the run's date and commit, and the last complete run. The
  run also adds a `complete: false` row and exits 0. functions.md and the SVGs are left alone.
  Catalog mismatches and malformed reports or markers still exit non-zero. When there is no
  existing page, the other generated blocks get a placeholder. Tests cover each reason, and
  test_progress_history.py also covers the byte-identical rest of the page.
- A report whose `run.commit` is not HEAD still counts as complete. After review, a complete page
  names only the report's commit and never HEAD, so the same report regenerates it byte for byte
  from any checkout (test). The commits in "since the last run" are counted between the two runs'
  own commits (`<last row's commit>..<run.commit>`), not up to HEAD. A history row's commit is
  `run.commit`, and falls back to HEAD only when the report has none. Only a Stale banner names the
  generating HEAD as well.
- The history row's `milestones_exited` lists the report's `milestone_exit` rows with `exit: true`.
  The report has rows for M5 and M6 only, and neither has exited, so the list is empty today.
  T-0072 parses docs/milestones.md and may widen this.
- The seed is one row for the T-0081 run (2026-10-25, `05703c6c`, complete), regenerated from
  the same report.json as T-0081's committed page, with tasks.json regenerated on this branch. No
  past rows were invented. Apart from the new blocks and method text, the regenerated page is the
  T-0081 page on `main`. An earlier regeneration on this branch showed probes known failures as
  2,686 where `main` shows 2,666. That was drift in eval/baseline/probes.txt from `main` (the
  branch's baseline before the merge), not this change; after the merge the page matches `main`
  again. A second regeneration leaves `git status` clean.
- Review fixes:
  - the tier sentence in parity.py that the merge of `main` dropped is restored;
  - an incomplete run older than the last history row still rewrites the status block as stale
    and exits 0, but its row is not recorded and the reason is printed (test);
  - `RUN_DATE` and the git commit count take ASCII digits only (test);
  - SPEC-0022 and the method text now say that an incomplete rerun never replaces a complete row
    of the same date;
  - a complete page no longer depends on HEAD (see above; SPEC-0022 updated to match);
  - `main` delegates to `_stale_outcome` and `_complete_outcome`, and svg.py's `_runs` yields
    (index, value) pairs.
- Gates: `make py-check` (1011 passed), `make hygiene`, `scripts/check_governance.py origin/main`
  and `make typos`.
