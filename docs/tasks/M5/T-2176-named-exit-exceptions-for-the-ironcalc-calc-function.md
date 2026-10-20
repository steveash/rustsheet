---
id: T-2176
title: "Named exit exceptions for the ironcalc-calc function-scope rows (eval/milestones/exit-exceptions.tsv)"
milestone: M5
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The "Milestone function scope (ironcalc-calc)" rows report how many failing cells remain once
the exceptions that decisions name are left out. Decisions 0074 and 0085 name IF_ARRAY C11, C4
and E4, and decision 0085 item 3 lets a later PR name time-boxed cells. These exceptions live in
one machine-read, protected file, not only in the prose of docs/milestones.md. The M5
ironcalc-calc criterion ("failing count is 0 once the exceptions are left out", decision 0085
item 1) can then be read from report.md without hand arithmetic.

## Context
- Decision 0085 items 1–3: the M5 row of "Milestone function scope" must reach 0 failing, leaving
  out `ironcalc-calc/LOGICAL/IF_ARRAY.xlsx` Sheet1!C11 (decision 0074, M2 onwards), Sheet1!C4 and
  Sheet1!E4 (decision 0085, M5 onwards; their filed task is the M14 item T-1075). Item 3 also lets
  the PR that blocks an M5 fit task under decision 0056 name cells within 4 ulp. Today these are
  named only in docs/milestones.md (M2 and M5 exit text), so the steward subtracts them by hand.
- Code: `crates/rustsheet-eval/src/function_scope.rs` (`summarize`, `write_markdown`; failing ids are
  `format!("{}!{}", r.id, c.key())`, e.g. `ironcalc-calc/LOGICAL/IF_ARRAY.xlsx!Sheet1!C11`). Use the
  parsing and validation pattern of `crates/rustsheet-eval/src/probe_milestones.rs` (the
  `probe-milestones.tsv` reader) and `milestone_lists.rs`. Do not write a second TSV reader if one
  can be shared (CLAUDE.md directive 3).
- `eval/milestones/` is a protected path (scripts/check_governance.py, decisions 0043, 0044), so
  the PR needs a decision record (`make new-decision`). That is deliberate: every line hides a
  failing cell from an exit measure, so adding one must take a decision. The file never touches
  the gate. The cells stay F lines in eval/baseline/ironcalc-calc.txt, and `eval-check` is
  unchanged.
- This needs judgment, so it is opus: what counts as stale, how a cumulative row inherits a line,
  and wording the decision so it weakens no gate.

## Acceptance criteria
- [ ] `eval/milestones/exit-exceptions.tsv` exists. It has `#` comment lines that explain the
      format and cite decisions 0074 and 0085. Each data line is
      `cell<TAB>milestone<TAB>decision<TAB>task`, for example
      `ironcalc-calc/LOGICAL/IF_ARRAY.xlsx!Sheet1!C11	M2	0074	T-1075`. It holds exactly the three
      IF_ARRAY lines (C11 from M2, C4 and E4 from M5).
- [ ] `run` and `check` parse the file and fail, naming the line, on four kinds of bad line: a
      malformed line, a milestone that is not `M\d+`, a decision that is not four digits, or a
      duplicate cell. A missing file means no exceptions.
- [ ] Each `MilestoneScope` row gains `excepted: Vec<String>`, the row's failing cells that carry an
      exception whose milestone is ≤ the row's. The rows also gain a `failing excl. exceptions`
      column in report.md (report.json: `function_scope.rows[].excepted`). `failing` keeps its
      current meaning and value.
- [ ] An exception whose cell is not failing in any row (it passes, is a deviation, or is not
      scored) is listed as stale under the section ("Stale exit exceptions"), the way
      `stale_deviations` are reported. It does not fail the run.
- [ ] Unit tests cover four cases: an exception that applies from its milestone onwards but not to
      an earlier row, a stale exception, each parse error, and an absent file.
- [ ] On `make eval` at the PR's base, the M2 row's `failing excl. exceptions` is 0, and the M5 row's
      equals its `failing` minus 3. The PR text quotes both rows.
- [ ] A new decision record (`make new-decision`) says the file holds only exceptions that a
      decision names, never hides a cell from the baseline or the gate, and that a line is added
      in the PR that takes the decision. docs/milestones.md's M2 and M5 exit text points to the
      file. docs/architecture/eval-harness.md "Milestone function scope" describes it.
- [ ] `scripts/check_governance.py origin/main` passes with the decision record. `make check`
      green; reviewers approve.

## Out of scope
- Adding any exception beyond the three IF_ARRAY cells. ATAN2_POWER D14 and D26 are T-2142's call.
  Time-boxed fit cells are added by the PR that blocks their task (decision 0085 item 3).
- Exceptions for suites other than ironcalc-calc, or for the probe and gen-m5 rows. Decision 0085
  gives those none.
- The combined exit section (T-2177).
