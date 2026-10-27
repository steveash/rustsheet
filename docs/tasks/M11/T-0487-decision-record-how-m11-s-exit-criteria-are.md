---
id: T-0487
title: "Decision record: how M11's exit criteria are measured (corpus-bench formula-cell match against the published leaderboard, M11 probes)"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A decision record (`make new-decision`) fixes how M11's exit criteria are read from `make eval`'s report at visible
parity (decision 0114), so an M11 exit row and `make exit-owners MILESTONE=M11` can be built from it. It adds no gate
and changes no baseline.

## Context
- M11's exit text (docs/milestones.md "M11"): "formula-cell match on xlsx-corpus-bench ≥ 99.8%, matching or beating
  the best published engine; its `probes` cases (decision 0051) at 100% bit-exact". `MILESTONES` in
  crates/rustsheet-eval/src/milestone_exit.rs stops at M6.
- Evidence so far: T-0427 (done) filled corpus-bench from excel-graph and compared against xlsx-corpus-bench's
  published rates (LibreOffice 96.7–98.9%, witan 99.7–99.8%; the leaderboard's comparator uses a 1e-9 relative
  tolerance). docs/progress.md (run of 2026-10-27, 7bd9090e): corpus-bench 912 workbooks, 205,366 cells, 99.83%
  bit-exact, a visibly-off cell in 2 of 320 scored workbooks. Its workbooks are fetched, not committed
  (`rustsheet-eval fetch --suite corpus-bench`); pending workbooks are listed in eval/suites/corpus-bench/PENDING.md.
- "Probe milestones (decision 0051)" M11 row on 2026-10-27: 4 cases, 32 scored, 0 known failures. The data-table and
  iterative-calculation cases need the Phase 2 graph for their implementation (docs/milestones.md "M11").
- Questions the record answers:
  1. Which corpus-bench cells count ("formula-cell match": formula cells only, Excel-backed sidecar cells only), and
     how deviations, pending and web-vs-desktop conflict cells (decision 0019) are treated.
  2. Which comparator reads the 99.8%: the leaderboard's 1e-9 relative tolerance, decision 0114's 15 digits, or both
     reported with one required; "matching or beating the best published engine" as a fixed number (cite the
     manifest version) or as a report column.
  3. Whether poi, calipers or the FUSE subset (T-0431, blocked) join the measure or stay information.
  4. How the row behaves when corpus-bench was not fetched (`n/a`, never met), as milestone_exit.rs does for absent
     inputs.

## Acceptance criteria
- [ ] Decision record added answering questions 1–4 with a current report's numbers (fetch corpus-bench first);
      docs/milestones.md's M11 exit text cites it (edit only the M11 section).
- [ ] A follow-up harness task is filed with `make new-task` for the M11 exit row and exit-owners support, with concrete
      thresholds.
- [ ] `python3 scripts/tasks.py validate` passes; `make check` green; reviewers approve.

## Out of scope
- Implementing the exit row (follow-up task).
- Data tables and iterative calculation (Phase 2).
- Fixing any corpus-bench cell.
