---
id: T-2890
title: "Decision record: how M9's exit criteria are measured at visible parity (ironcalc-calc overall, dense sweeps per numeric function, M9 probes)"
milestone: M9
lane: research
status: dropped
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A decision record (`make new-decision`) fixes how each of M9's exit criteria is read from `make eval`'s report at
visible parity (decision 0114), so an M9 exit row and `make exit-owners MILESTONE=M9` can be built from it. It adds no
gate and changes no baseline.

## Context
- M9's exit text (docs/milestones.md "M9"): "ironcalc-calc at ≥ 99.95% bit-exact overall (from 97.33%); dense oracle
  sweeps per numeric function at ≥ 99.9% bit-exact and 100% within 4 ULP; its `probes` cases (decision 0051) at 100%
  bit-exact". Decision 0114 reads "bit-exact" in Phase 1 as "not visibly off", and moves the bit-level and 4-ulp
  targets to Phase 1b. None of the criteria says which cells count. `MILESTONES` in
  crates/rustsheet-eval/src/milestone_exit.rs stops at M6.
- Planner run of 2026-10-27 (`make eval` on 6087413bc; poi, calipers and corpus-bench not fetched):
  - ironcalc-calc: 72,958 cells, 72,335 bit-exact (99.15%), 99.95% within 1e-9. "Milestone function scope" M9 row
    (cumulative, every catalog milestone up to M9): 623 failing, 618 excluding exit exceptions.
  - gen-m9: 264 cases, 96,462 cells, 96.11% bit-exact. probes-numeric: 612,244 cells, 57.22% bit-exact, 99.94% within
    1e-9, `gate = false`, tagged M1. Many `probes` cases are per-function sweeps (`*-sweep`, T-2046, T-2742 and others).
  - "Probe milestones (decision 0051)" M9 row: 490 cases, 38,196 scored, 2,658 known failures, 862 visibly off.
  - The report's "Fifteen-digit parity checkpoint" (`v15_checkpoint`, decision 0118) already gives each function's
    share of attributed cells within 15 digits.
- Questions the record answers:
  1. "Overall" for ironcalc-calc: every cell of the suite, or the M9 function-scope row; how the 99.95% reads at
     visible parity, and which exit exceptions (eval/milestones/exit-exceptions.tsv) carry over.
  2. What a "dense oracle sweep" is (gen-m9, probes-numeric, the sweep probe cases, or decision 0118's attributed
     cells per function) and which functions are "numeric functions" (catalog milestone M9, or every M9 category).
  3. The per-function threshold at visible parity (≥ 99.9% within 15 digits, as decision 0114's Phase 1 rate) and
     whether it reuses `v15_checkpoint` rather than a second count (CLAUDE.md directive 3).
  4. Which cells `exit-owners` lists for each criterion, given the M9 probe row alone has 862 visibly-off cells.

## Acceptance criteria
- [ ] Decision record added answering questions 1–4 with the 2026-10-27 (or newer) report numbers; docs/milestones.md's
      M9 exit text cites it (edit only the M9 section).
- [ ] A follow-up harness task is filed with `make new-task` for the M9 exit row and exit-owners support, with concrete
      thresholds; any report measure the record requires that does not exist yet is filed too.
- [ ] `python3 scripts/tasks.py validate` passes; `make check` green; reviewers approve.

## Out of scope
- Implementing the exit row (follow-up task).
- Phase 1b's bit-level and 4-ulp targets beyond naming where they will be read.
- Fixing any M9 cell.

## Outcome (dropped, 2026-10-28, T-0088)
Superseded by decision 0146 (T-0088), which answers this task's questions for M1 and M8 to M11 in one record and
cites it in docs/milestones.md. The follow-up exit row and exit-owners support are T-0089's, which lists the
thresholds.
