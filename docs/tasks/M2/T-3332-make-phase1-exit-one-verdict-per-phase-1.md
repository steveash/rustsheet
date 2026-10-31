---
id: T-3332
title: "make phase1-exit: one verdict per Phase 1 exit criterion of docs/milestones.md, from report.json and the task board"
milestone: M2
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
One command, `make phase1-exit`, says whether Phase 1 has exited: it prints one line per exit
criterion of docs/milestones.md "Phase 1 exit (decision 0114)", each `met`, `not met` or
`manual` with the numbers behind it, and an overall verdict. It exits 1 unless every measured
criterion is met. The steward declares the exit from it, not from a reading of the prose.

## Context
- Today the steward skill (.claude/skills/steward/SKILL.md, "Phase 1 exit") adds
  `**Phase 1:** exited <date>` when `v15_checkpoint.phase1_met` is true "and the rest of
  docs/milestones.md's Phase 1 exit holds". Nothing measures that rest. The declaration also
  removes every `defer: phase-1b` key and unblocks Phase 2 (M3, M4, M13), so declaring it early
  is costly to undo.
- `phase1_met` counts only the per-function rate and coverage (crates/rustsheet-eval/src/v15_checkpoint.rs).
  The other criteria are in several places:
  1. **Every Excel function is in the catalog with its parity scope (T-0040).** eval/catalog/excel-functions.tsv
     against eval/catalog/functions.yaml and the report's `out_of_scope` list.
  2. **Coverage and rate per function.** report.json `v15_checkpoint.phase1_met`. The
     `stale_exceptions` and `unchecked_exceptions` there must be empty: a stale or unchecked
     exception line means the rate was computed against a wrong exception list.
  3. **Each remaining failure has a filed task or a `DEVIATION` entry.** `scripts/exit_owners.py --all-off`
     (attributed, unattributed and probe-row cells; a cell owned only by a `defer:` task counts as unowned)
     and `--milestone M<k>` for each Phase 1 milestone (decision 0096).
  4. **The exit criteria of M1, M2, M5-M10 and M11 are met.** report.json `milestone_exit[].exit`
     covers the measured part. The prose items checked outside the rows (decision 0146: M1's
     parity bugs and API v0; M2's one-walker/one-matcher criterion, met at 11592ab5 per decision 0137)
     are not measurable from the report: list them as `manual` with their docs/milestones.md line.
  5. **Every suite ran.** report.json `run.skipped_suites` is empty and `run.dirty` is false. A
     run that skipped calipers, corpus-bench or poi reads M8's poi subset and M11 as `n/a`, and
     its owner counts miss their cells.
- Seen on 2026-10-31 at 9453cfb5 (planner run): with calipers, corpus-bench and poi not fetched,
  `--all-off` printed 23 unowned cells; after `rustsheet-eval fetch` and a rerun it printed 0. In
  the same full run `exit_owners.py --milestone M11` listed 6 corpus-bench/13894 cells
  (Sheet1!F20:F25) that only M9 task T-0950 names, while the M11 exit row reads `met` (6 visibly
  off within the 410 allowance). A checker has to report both facts and not hide one behind the other.
- `scripts/` and `.claude/skills/` are protected paths (`check_governance.py`), so this PR needs a
  decision record (`make new-decision`) that fixes the criterion-to-measure mapping above, and
  approval from `reviewer-architecture`.

## Acceptance criteria
- [ ] A decision record maps each Phase 1 exit criterion of docs/milestones.md to its measure
      (or to `manual`) as listed in Context, including how the M11 allowance case above is reported.
- [ ] `scripts/phase1_exit.py` (target `make phase1-exit`, optional `--report PATH --tasks DIR --json`)
      prints one line per criterion and an overall line. It exits 1 when any measured criterion is
      not met or the report is from a dirty or partial run, and 0 otherwise. It reuses
      `exit_owners.py`'s functions by import, with no second ownership rule, and reads the
      exit and rate verdicts from report.json without recomputing them.
- [ ] `scripts/test_phase1_exit.py` has fixture reports and task trees for: all met (exit 0); one
      milestone row not met; `phase1_met` false; a skipped suite; one unowned cell; one cell owned
      only by a `defer: phase-1b` task; a non-empty `stale_exceptions`. Each failing fixture names
      the failing criterion in the output.
- [ ] Run on a full `make eval` of the PR's base (all 17 suites fetched), it prints `not met` and
      names M2, M6, M7, M8, M9 and M10 and the function-rate criterion as not met. The PR
      description pastes that output.
- [ ] The steward skill's "Phase 1 exit" rule says to declare the exit only when `make phase1-exit`
      exits 0 and every `manual` line has been checked, and docs/architecture/eval-harness.md
      lists the command.
- [ ] `make check` green; reviewers approve (`reviewer-architecture` for the protected paths).

## Out of scope
- A Phase 1 line in the progress dashboard's goals block (tools/ progress_report). File it as a follow-up
  that reads `--json`.
- Changing any exit threshold, allowance or ownership rule.
- Owning the 6 corpus-bench/13894 cells: that is a planning decision for M11 (T-0950's milestone).
