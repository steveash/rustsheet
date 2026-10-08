---
id: T-1146
title: "Report the known failures of a milestone's frozen probe list (eval/milestones/m1-probes.txt)"
milestone: M1
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
`make eval` reports, for each milestone probe list in `eval/milestones/` (today only
`m1-probes.txt`, decision 0043), how many cases and scored cells it has, how many of them pass
bit-exact, how many are deviations and how many are known failures. The steward then reads M1's
probe criterion from the report instead of counting F lines by hand, and a list that names a
missing case fails the gate.

## Context
- Decision 0043 freezes M1's probe criterion as the 89 cases in `eval/milestones/m1-probes.txt`.
  The file has `#` comment lines, then one case id per line in the form of eval/baseline/probes.txt
  (`probes/<path>.case.json`). The rest of the `probes` suite is M2 exit scope.
- No harness command reports a listed subset of cases. `--filter` takes a single substring.
  Until this lands, the steward uses the `awk` count in decision 0043, item 4: 61 M1-list F lines
  and 136 others at the freeze commit e5c7ae9.
- Code: `crates/rustsheet-eval/src/report.rs` (`RunReport`, `SuiteSummary`, report.md writer),
  `crates/rustsheet-eval/src/main.rs` (`Run`, `Check`), and `crates/rustsheet-eval/src/deviations.rs`.
  deviations.rs is the model for loading a repo data file with a `thiserror` error.
- The decision says a PR that edits `eval/milestones/` needs a decision record.
  `scripts/check_governance.py` does not enforce that yet.

## Acceptance criteria
- [ ] A new module (for example `crates/rustsheet-eval/src/milestone_lists.rs`) parses
      `eval/milestones/*.txt`: skip `#` and blank lines, reject duplicates, and use a `thiserror`
      error type. Unit tests cover comments, duplicates and an unknown case id.
- [ ] `make eval` writes a "Milestone probe lists" section to eval/runs/latest/report.md, and the
      same data to report.json. It has one row per list, with these columns: cases, scored cells,
      bit-exact, deviations, known failures (cells), and the failing case ids. Run on main at the
      freeze commit, the m1-probes row shows 89 cases and 61 known-failure cells, the same as the
      decision's `awk` count.
- [ ] `make eval-check` fails, naming the line, when a list names a case that the suite does not
      discover.
- [ ] `scripts/check_governance.py` treats `eval/milestones/` as a protected path, with a test in
      `scripts/test_*.py`.
- [ ] docs/architecture/eval-harness.md describes the lists and the report section, and the
      `awk` interim note in the steward skill (`.claude/skills/steward/SKILL.md`, "Frozen probe
      lists") points to the report instead.
- [ ] `make check` green; reviewers approve (architecture and security for the protected paths).

## Out of scope
- Changing the list's contents, any case file or tag, or eval/baseline/ (decision 0043).
- Gating on the list's pass rate. The list is an exit measure, not a CI gate; regressions are
  already caught by `eval-check`.
- Lists for other milestones (filed when a decision creates one).
