# 0120 — make progress-report replaces make parity-report, and the progress page's docs inputs count as code

- **Status:** accepted (2026-10-25, T-0070; SPEC-0022). Items 1, 2 and 4 completed by T-0081.
- **Amends:** decision 0079 (the `make parity-report` target and "nothing under docs/ is read by
  code"); extends decision 0066's `CODE_READ_PATHS`.
- **Context:**
  - SPEC-0022 replaces the parity page with a daily progress dashboard, docs/progress.md. Its
    generator is the parity generator, renamed into the package
    `tools/src/rustsheet_tools/progress_report/` with `git mv`.
  - The new generator reads three docs paths:
    - docs/progress.md, to keep the steward's judgment blocks;
    - docs/progress/history.json, for the deltas and trends (T-0071);
    - docs/milestones.md, for the milestone exit rows (T-0072).
  - Decision 0079 said nothing under docs/ is read by code. Decisions 0049 and 0066 skip the rust,
    eval and python CI jobs for PRs that change only docs/. `scripts/check_docs_readers.py` fails
    on a docs literal in tools/ unless its path is in `CODE_READ_PATHS`.
  - The Makefile and scripts/ci_changes.py are protected paths.
- **Decision:**
  1. **The target.** `make progress-report` replaces `make parity-report`. It does two things:
     - it writes `eval/runs/latest/tasks.json` with `scripts/tasks.py stats --json` (decision
       0119), through a temporary file, so a failed command leaves no partial file (T-0081: the
       target also removes the temporary file when the command fails);
     - it runs the entry point `rustsheet-progress-report --out-dir ../docs`, which replaces
       `rustsheet-parity-report`.

     The `COMMIT=` and `RUN_DATE=` arguments go away. The run's date and commit now come from
     report.json `run` (T-0080), checked before they are printed.
  2. **Docs inputs are code.** `CODE_READ_PATHS` in scripts/ci_changes.py gains
     `docs/milestones.md`, `docs/progress.md` and `docs/progress/`. A PR that changes only one of
     them, such as a steward PR that changes only docs/progress/history.json, is `code=true` and
     runs the python job (scripts/test_ci_changes.py). The generator may name these paths as
     literals. An entry ending in `/` is a directory and matches every path under it; any other
     entry matches only that exact file (T-0081), so docs/progress.md does not also claim, say,
     docs/progress.md.bak. `scripts/check_docs_readers.py` applies the same rule.
  3. **What stays from 0079.** The generated parts of the page are never hand-edited. The page
     names the run it shows, so it may lag `main`. No CI freshness gate checks docs/progress.md,
     docs/progress/ or docs/parity.md: a gate would fail every PR that moves a number.
  4. **docs/parity.md.** Until T-0081, the target still writes docs/parity.md and docs/parity/
     as before. T-0081 then turns docs/parity.md and docs/parity/functions.md into short static
     pointers to the progress page, deletes the three docs/parity/*.svg charts, and stops writing
     either file: the target writes only docs/progress.md and docs/progress/.
- **Consequences:**
  - The steward runs `make progress-report` instead of `make parity-report` (T-0076).
  - A PR that changes only docs/milestones.md, docs/progress.md or docs/progress/ now runs the
    rust, eval and python jobs, which costs CI time. Decision 0049 asks that of every path code
    reads: a skipped job looks exactly like a pass. Outside the daily steward PR, such PRs are
    rare.
  - CI classifies with the base commit's copy of scripts/ci_changes.py (decision 0066). The new
    paths count as code for PRs opened against the commit that merges this one.
