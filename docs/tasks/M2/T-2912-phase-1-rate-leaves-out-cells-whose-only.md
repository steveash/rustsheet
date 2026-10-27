---
id: T-2912
title: "Phase 1 rate leaves out cells whose only open question is desktop evidence (decision 0039): AND and MODE miss it only on Excel 15 poi caches"
milestone: M2
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The Phase 1 rate (decision 0118, report.json `v15_checkpoint`) leaves out the cells that a decision assigns to M14
because their only open question is desktop Excel evidence, as docs/milestones.md requires: such a failure "never
blocks the exit of Phase 1, any other phase or any milestone" (decision 0039). M2's AND, and MODE, then meet the
rate on the cells Phase 1 can actually settle. The left-out cells stay known failures and stay visible in the report.

## Context
- `make eval` on 179909a8 (all 17 suites, calipers, corpus-bench and poi fetched): AND has 466 attributed cells.
  465 pass and 1 is visibly off, 99.79%, under the 99.9% rate. The current milestone has one function below the
  Phase 1 rate, and this is it. The one cell is poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J136
  `AND("1",1)`: Excel 15 cached `#VALUE!`, the engine gives TRUE as Excel for the web does (probes/and-or-xor-text-arguments,
  T-0258). docs/excel-semantics/coercion.md "AND/OR/XOR text arguments" records it as CONFLICTING (decision 0030
  rule 2: AppVersion below 16 does not outrank the web), and T-1075 item 7 (M14) owns it.
- MODE (M9) is in the same position: 338 of 341 attributed cells pass, and the 3 visibly-off cells are
  EverythingTests!G936, H936 and K936 of the same workbook. Each is an Excel 15 cached `#VALUE!` against
  Microsoft 365's `#N/A`/`#DIV/0!`, kept as known failures by decision 0067 until T-1075 decides. docs/progress.md
  lists both AND and MODE as functions the semantic fixes would lift, but no semantic fix exists for either.
- The ironcalc-calc milestone function-scope rows already have this mechanism: eval/milestones/exit-exceptions.tsv
  (decision 0087; crates/rustsheet-eval/src/exit_exceptions.rs). It names cells, the decision and the owning task,
  and hides nothing from the baseline or the gate. It only reads ironcalc-calc cells and only the function-scope
  rows. The Phase 1 rate is computed in crates/rustsheet-eval/src/v15_checkpoint.rs (`summarize`, attributed
  counts from attribution.rs) and has no exception list.
- Design question for the decision record: extend exit-exceptions.tsv to any suite and to the Phase 1 rate (with a
  column or a milestone value that says which measures leave the cell out), or add a sibling file under
  eval/milestones/. Either way, each line needs a decision that names the cell as M14-only and an open M14 owner.
  The existing IF_ARRAY and ATAN2_POWER lines are Mac desktop caches that T-1075 owns. Whether they also leave the
  rate is part of the record.
- Protected paths: eval/milestones/ (decision 0044) and any scripts/ change need the decision record in the same
  PR. The progress dashboard (tools/src/rustsheet_tools/progress_report/) reads `v15_checkpoint`. Check that it
  still renders and that its "lifted by the semantic fixes" list stops naming functions whose only off cells are
  left out.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) amends decisions 0087 and 0118. It defines which cells leave the
      Phase 1 rate (assigned to M14 by decision 0039, with an open M14 owner) and how they are listed. It names
      poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J136, G936, H936 and K936 as the first such cells, citing
      decisions 0030, 0039 and 0067. The architecture reviewer approves it.
- [ ] After `make eval` with poi fetched, AND and MODE have `meets_rate` true in report.json `v15_checkpoint`, and
      report.md shows the number of left-out cells next to the Phase 1 line (or per function). The four cells
      stay F lines in eval/baseline/poi.txt (`make eval-baseline` changes nothing for them).
- [ ] Unit tests in v15_checkpoint.rs: a listed cell does not count toward a function's rate. An unlisted
      visibly-off cell still does. A listed cell that passes is ignored. A listed cell that is not scored or not
      failing is reported (warning or error, as the record says), so the list cannot go stale silently.
- [ ] The harness rejects a line with no decision or no `T-` owner (same validation as exit_exceptions.rs). The
      ironcalc-calc function-scope rows keep their current numbers.
- [ ] The Outcome lists every function whose `meets_rate` changed and the Phase 1 count before and after
      (377 functions on the 2026-10-27 dashboard).
- [ ] `make check` green; `make progress-report` still renders; reviewers approve.

## Out of scope
- Changing any engine result, or overriding the cached values (decision 0067 rejected `cached_overrides` for
  Excel-version conflicts).
- Listing other conflict cells beyond the four named above. Each needs its own decision citing its evidence; the
  steward or gap-triager files them.
- Decision 0094's coverage count and the strict information count (decision 0118 item 5) stay as they are.
