# 0134 — Phase 1 rate leaves out cells listed as M14 desktop-only evidence

- **Status:** accepted (2026-10-27, T-2912).
- **Amends:**
  - decision 0087: the exit exceptions keep their file and their measure (the ironcalc-calc
    function-scope rows); the Phase 1 rate gets a sibling list under the same rules;
  - decision 0118 item 3: a function's Phase 1 rate leaves out the cells this list names.
- **Context:**
  - Decision 0039 rule 3: a failure whose only open question needs desktop Excel evidence has its
    task in M14 and "never blocks the exit of Phase 1, any other phase or any milestone"
    (docs/milestones.md). The Phase 1 rate of `v15_checkpoint` (decision 0118) had no way to
    leave such a cell out, so it still counted against its function.
  - `make eval` on 179909a8 (all 17 suites): AND has 466 attributed cells, 1 visibly off
    (99.79%, under the 99.9% rate). The cell is poi/FormulaEvalTestData_Copy.xlsx
    EverythingTests!J136 `AND("1",1)`: Excel 15 cached `#VALUE!`, Microsoft 365 (excel-graph,
    probes/and-or-xor-text-arguments) gives TRUE, as the engine does. coercion.md "AND/OR/XOR
    text arguments" records it as CONFLICTING; T-1075 item 7 (M14) owns it.
  - MODE has 341 attributed cells, 3 visibly off: EverythingTests!G936, H936 and K936 of the same
    workbook. Excel 15 cached `#VALUE!`; Microsoft 365 gives `#N/A`, `#N/A` and `#DIV/0!`
    (probes/mode-typed-text-no-mode). Decision 0067 keeps them as known failures until T-1075
    item 10 (M14) decides.
  - Decision 0030 rule 2: an Excel build below 16 does not outrank the web, so in Phases 1–3 the
    engine keeps the web value and the four cells stay off. No fix exists to make in Phase 1.
- **Options considered:**
  - *Extend exit-exceptions.tsv to any suite with a column naming the measure.* Every existing
    line would change, and its milestone column ("the first function-scope row that leaves the
    cell out") means nothing for the Phase 1 rate, which has no rows.
  - *Override the cached values (`cached_overrides`).* Decision 0067 rejected it for
    Excel-version conflicts: the cells must stay visible as known failures.
  - *A sibling list for the Phase 1 rate (chosen).* Same validation and the same "hides nothing"
    rule, one measure per file.
- **Decision:**
  1. `eval/milestones/phase1-exceptions.tsv` lists the cells the Phase 1 rate leaves out. A line
     is `case!Sheet!A1` (any suite), the decision that names the cell as M14-only, and the open
     M14 task that owns it. A cell is listed only when a decision shows that its only open
     question is desktop Excel evidence (decision 0039) and names it; the line is added in that
     decision's PR (eval/milestones/ is protected, decision 0044).
  2. A listed cell that is a visibly-off, non-deviation Excel cell of the run leaves the Phase 1
     count of each function whose counted cells hold it (decision 0118: its sole non-glue call,
     or every function it calls for a function measured by the fallback). It is reported as
     `rate_excepted`; `rate_cells` and `rate_visible` drop by it. Which count a function uses
     (attributed or fallback) is decided before the cell leaves.
  3. It hides nothing else. The cell stays an F line of eval/baseline, the gate is unchanged, and
     the function's `cells`, `visible`, tier, strict count (decision 0118 item 5) and decision
     0094's checkpoint (coverage and the zero-visibly-off count) keep it.
  4. report.md shows the number of left-out cells per function under the Phase 1 line. A listed
     cell that the run does not score, or that is not visibly off, is listed as stale with a
     warning; like decision 0087's stale lines it never fails the run, and its line should go.
  5. `rustsheet-eval run` and `check` refuse a malformed line: no decision (four digits), no
     `T-<digits>` owner, a malformed cell, or a repeated cell.
  6. The first cells are poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J136 (decisions 0030,
     0039; T-1075 item 7) and G936, H936 and K936 (decisions 0030, 0039, 0067; T-1075 item 10).
  7. The exit-exceptions.tsv cells (IF_ARRAY C4, E4, C11 and ATAN2_POWER D14, D26) are not listed:
     they are within 15 significant digits (one ulp from the web), so they are not visibly off
     and never counted against the rate.
- **Consequences:**
  - AND and MODE meet the Phase 1 rate once poi is fetched and scored; the progress dashboard
    stops naming them among the functions the semantic fixes would lift.
  - The steward or gap-triager may propose more cells, each in a PR with its own decision citing
    its evidence. Removing a line (the cell passes, or desktop evidence settles it) needs no
    decision.
  - When T-1075 runs (Phase 4), every line it settles goes: fixed cells pass, and cells desktop
    Excel confirms become ordinary failures to fix.
