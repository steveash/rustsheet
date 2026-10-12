---
id: T-1423
title: "Report probe known failures per assigned milestone (decision 0051)"
milestone: M2
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
`make eval` reports, for each milestone that decision 0051 assigns probe cases to, how many cases
and scored cells it has, how many pass bit-exact, how many are deviations and how many are known
failures. Rows: M0–M2 combined, then M5, M6, M7, M8, M9, M10 and M11. The steward then reads M2's
probe criterion, and each later milestone's, from the report rather than from the decision's
one-liner.

## Context
- Decision 0051 gives every `probes` case one milestone. For `probes/catalog/*`, it gives one
  milestone per checked cell instead. The first rule that applies decides:
  1. A case on `eval/milestones/m1-probes.txt` is M1.
  2. A case listed in `eval/milestones/probe-milestones.tsv` takes the milestone given there.
     Each line is tab-separated: case id, milestone, subject. `#` starts a comment.
  3. A `probes/catalog/*` cell takes the highest `eval/catalog/scope.json` milestone of the
     functions its formula calls (`NAME(`), apart from OR, IFERROR, ERROR.TYPE and INDEX. A cell
     that calls none is M2.
  4. Any other case takes its case.json milestone tag (`M\d+`; the highest one if a case has
     several).
  5. A case with no tag is M2.
- T-1146 (done) added the per-list report: `crates/rustsheet-eval/src/milestone_lists.rs`, the
  "Milestone probe lists" section of report.md and `milestone_lists` in report.json. It loads only
  `eval/milestones/*.txt`, so the `.tsv` is not read today. The new report reuses that scoring
  (`Baseline::from_results`), so its known failures equal the F lines that the decision's
  one-liner counts.
- The milestone of each catalog function is already loaded from `eval/catalog/scope.json`
  (decision 0040). Search for the existing loader with `rg scope.json crates/rustsheet-eval`, and
  reuse it.
- Numbers at origin/main 9ad32ae1, from the one-liner in decision 0051:
  `M0-M2=74 M5=35 M6=175 M7=1547 M8=3 M9=994 M10=53 M11=0 total=2881`.

## Acceptance criteria
- [ ] The mapping file accepts an optional per-cell form (`case id`, `Sheet!A1`, milestone, subject)
      that overrides the case-level milestone for that cell only; move
      `subtotal-reference-operands-d` Sheet1!H28 (`=GEOMEAN(5,7)`) to M9 with it (decision 0051,
      "Known consequences"). Record the change in decision 0051 (governed path).
- [ ] The harness parses `eval/milestones/probe-milestones.tsv` with a `thiserror` error type.
      Unit tests cover:
      - a comment line;
      - a duplicate case;
      - a milestone that is not `M\d+`;
      - an unknown case id. `run` and `check` fail on it, naming the line, as for the lists.
- [ ] Unit tests cover each of rules 1–5: a frozen-list case that is also tagged later stays M1;
      an override line beats the tag; a catalog cell wrapped in OR/IFERROR/ERROR.TYPE counts at
      its function's milestone; a catalog cell that calls no catalog function is M2; a case with
      no tag is M2.
- [ ] report.md gets a "Probe milestones (decision 0051)" section, and report.json gets the same
      data. The section has one row each for M0–M2, M5, M6, M7, M8, M9, M10 and M11, with these
      columns: cases, scored cells, bit-exact, deviations and known failures (cells). The console
      summary prints the M0–M2 row. On the commit the PR branches from, the known-failure column
      equals the decision 0051 one-liner's output. The PR description quotes both.
- [ ] docs/architecture/eval-harness.md describes the section and the rules (citing decision
      0051). The steward skill's "Probe milestones" bullet points to the report section, not the
      one-liner.
- [ ] `make check` green; reviewers approve (architecture and security, for `.claude/`).

## Out of scope
- Changing `probe-milestones.tsv`, `m1-probes.txt`, any case file or tag, or `eval/baseline/`.
  Any of these needs a decision record (decisions 0043, 0044 and 0051).
- Gating CI on a milestone's pass rate. The rows are exit measures; `eval-check` already catches
  regressions.
- Per-milestone rows for suites other than `probes`.
