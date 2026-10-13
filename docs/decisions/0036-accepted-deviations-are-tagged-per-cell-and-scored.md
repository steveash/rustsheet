# 0036 — Accepted deviations are tagged per cell and scored apart from bit-exact

- **Status:** accepted (2026-10-06, T-0176)
- **Amended by:** decision 0052 (2026-10-13). A `deviations` file lists only directive-5 cells
  citing decision 0014. Other deviations do not exist; a cell RustSheet gets wrong for any other
  reason stays a known failure.
- **Context:**
  - The M1 exit asks for `smoke`, `probes` and `gen-m1` at 100% bit-exact, while the Phase 1 exit
    accepts "a filed task or a `DEVIATION` entry" for each remaining failure. The two disagree for
    cells that differ from Excel *by design* under decision 0014 (directive 5 beats directive 1).
  - The first such cells are probes/resource-limits C12 `=ROWS(SEQUENCE(1000,1001))` and C14
    `=ROWS(SEQUENCE(1048576))`. Excel gives 1000 and 1048576; RustSheet gives `#ERROR!` above
    1,000,000 array cells (docs/excel-semantics/arrays.md, "Array results above 1,000,000 cells are
    #ERROR!", DEVIATION). They are F lines in eval/baseline/probes.txt today, so they look like
    any other known failure. The steward cannot tell whether M1 can exit once T-0134 and T-0221
    land.
  - Other DEVIATION entries (the nesting cap in syntax.md, the text-length and surrogate limits in
    text.md and coercion.md, decision 0033) will have cells in later suites, including xlsx suites
    (ironcalc-calc, poi) whose cases are not JSON specs.
- **Options considered:**
  - *(a) Exit counts known failures that a DEVIATION entry cites, and the steward checks them by
    hand.* No tooling, but the check is manual and easy to get wrong. The baseline cannot tell an
    accepted deviation from a bug, so a deviation that silently changes value (for example
    `#ERROR!` becoming a wrong number) still reads as "known failure, unchanged". Kept only as the
    interim rule below.
  - *(b) Tag each deviation cell, score tagged cells in their own column, and pin their value.*
    Chosen.
  - *(c) Raise the limit until Excel's values fit.* `SEQUENCE(1048576)` alone is 1M cells, and
    `SEQUENCE(1048576,16384)` is 17G cells. No bounded design reaches Excel's sheet-sized arrays,
    and every new limit would reopen the question. The limit itself is out of scope (T-0176).
    Rejected.
  - *(d) Move the cells out of the gated probes suite.* This drops the evidence of what Excel does
    and stops pinning our behavior. It also needs editing cases to meet an exit number, which
    directive 2 forbids in spirit. Rejected.
- **Decision:**
  - A suite may have a `deviations` file, one line per cell, tab-separated:
    `<case path>  <Sheet!A1>  <decision NNNN>  <docs/excel-semantics/<file>.md#<anchor>>  <pinned value>`.
    It follows the format of decision 0032's `cached_overrides`. The pinned value is RustSheet's
    by-design result (`#ERROR!`). Excel's value stays the expectation from the oracle or cache, as
    for every cell.
  - The harness scores a listed cell as **deviation** when the engine gives the pinned value. It
    is a **failure** when the engine gives anything else, and a **pass** (with a warning that the
    line is stale) when the engine matches Excel. Deviation cells are left out of the bit-exact
    denominator and never appear as baseline F lines. The report shows a per-suite `deviations`
    column, and `eval-check` fails on a listed cell whose cited semantics entry has no
    `Status: DEVIATION` line.
  - Milestone and Phase exits that require "100% bit-exact" mean 100% of scored cells that are not
    listed deviations. Every listed deviation must cite an accepted DEVIATION entry.
  - Interim rule, until the harness work (T-0191) lands: the steward may treat a known failure as
    a deviation only if its F line is probes/resource-limits C12 or C14. Both cite the arrays.md
    DEVIATION entry. Before declaring M1 exit, the steward runs
    `make eval-show CASE=probes/resource-limits` and confirms both cells show `#ERROR!`, since the
    baseline does not pin their value. It then names both cells in docs/status.md.
- **Consequences:**
  - T-0191 (harness, M1, P1) implements the file, the scoring, the report column and the check.
    It moves C12/C14 from eval/baseline/probes.txt into `eval/suites/probes/deviations` with
    `make eval-baseline`, never by hand.
  - Adding a line to a `deviations` file is a protected change: it needs the DEVIATION entry
    (decision 0014) in the same PR, and the Excel-fidelity reviewer checks that the cited limit is
    a directive-5 limit, never a convenience.
  - Lifting a deviation (raising a limit until Excel's value fits) turns the cell into a pass. The
    harness warns, and the PR removes the line.
- **Implemented (T-0191, 2026-10-08):**
  - The harness side is `crates/rustsheet-eval/src/deviations.rs`, wired in through the suite
    manifest key `deviations = "<file>"`. A cell giving its pinned value gets the verdict
    `deviation`. The case's `deviations` count leaves the bit-exact rate, and the report has the
    per-suite column. `eval-check` and `eval` run the entry check before any case.
    docs/process/quality-gates.md and the steward skill describe the file.
  - probes/resource-limits C12 and C14 have passed since T-0047 raised the bound to one whole
    column (1,048,576 cells), so they are not listed. The first lines are
    probes/lifted-range-rows C1, C2, C3 and E1, pinned `#ERROR!`. They cite
    `docs/excel-semantics/arrays.md#array-results-above-1048576-cells-are-error`.
  - The interim rule above is retired. `make eval-baseline` dropped the four F lines from
    eval/baseline/probes.txt.
