# RustSheet and gridcraft compared (2026-10-29)

GitHub issue #2277 asked how RustSheet compares with [gridcraft](https://github.com/storytold/gridcraft),
another spreadsheet written in Rust, and whether its implementation has something to teach us (T-0496).

**Method.** gridcraft was read as data only, from a shallow clone of commit 2ff751791b7e (2026-10-29): its
README, `ROADMAP.md`, `AGENTS.md`, `docs/` and source files. Nothing of it was built or run. An
issue-origin task may not run third-party code (decision 0158), so unlike the mog comparison
(mog-head-to-head-2026-10-22.md) this note has no measured values or timings for gridcraft. A number
marked *(gridcraft's claim)* comes from its own docs and was not checked. RustSheet's numbers come from
docs/progress.md (eval run of 2026-10-28, commit 180c8194).

## Bottom line
- **Different products.** gridcraft is a desktop and web spreadsheet application (ribbon UI, charts,
  pivot tables, print, an MCP server), in alpha. RustSheet is a headless calculation engine whose only
  target is giving the same values as Excel.
- **Different evidence.** gridcraft works clean-room from public documentation and black-box
  observation of Excel. Its own function doc says it has "no conformance suite against Excel-computed
  results" and puts function depth at about 82% *(gridcraft's claim)*. RustSheet scores every cell
  bit for bit against oracle values from Excel in 17 suites.
- **Breadth is about equal.** gridcraft has 504 of RustSheet's 506 in-scope functions by name. Having
  a name says nothing about its values.
- **What to learn:** its interval index for range dependents is a useful input for RustSheet's
  dependency-graph spec (Phase 2), and three of the Excel behaviours its docs describe have no
  RustSheet probe yet (T-3119). Its numeric kernels are textbook approximations. Where RustSheet has
  evidence, Excel uses something else (AS 241 in the NORM.S.INV tails).

## What gridcraft is
- A Cargo workspace with enforced layers (README "Architecture", docs/architecture.md): `core` (values,
  coercion, dates), `numfmt`, `formula` (parser, printer, reference adjustment), `functions`, `model`
  (workbook, copy-on-write cell store), `calc` (evaluator, dependency graph, dynamic arrays), `xlsx`
  (XLSX read/write, CSV, XLSB and ODS import), `engine` (session, 313 commands), `ui-egui`, `mcp`.
- Size, counted on this commit (Rust lines including tests): functions 16.5k, calc 6.3k, xlsx 25.8k,
  engine 54k, ui-egui 67k. RustSheet's engine crate is about 250k lines, 76k of them under
  `functions/`.
- Licence MIT OR Apache-2.0 (README). The README says alpha and "about 50% ready for real work"
  *(gridcraft's claim)*.
- Not in RustSheet's scope at all: the UI, charts, printing, macros, the MCP server, `.xls` import.

## How each one checks Excel compatibility

| | gridcraft | RustSheet |
|---|---|---|
| Source of truth | Public documentation and black-box observation of Excel. Excel-produced files are never committed (AGENTS.md "Clean-room"). | Values computed by Excel (excel-graph oracle), committed as evidence (CLAUDE.md directive 1). |
| Automated check | `gridcraft-cli check` recalculates saved files and compares with their cached values: numbers within 1e-9 times the larger magnitude, or within an absolute 1e-9 below magnitude 1, and errors by kind (docs/cli.md; `apps/gridcraft-cli/src/check/calc.rs`). The corpus lives in a separate repository not named here. | 17 suites scored bit for bit, with ulp and 15-digit buckets, and a baseline that refuses regressions. |
| Function tests | 238 `#[test]` attributes in `crates/functions` and `crates/calc` (`grep -c '#\[test\]'`; its docs say 96 test functions in `crates/functions`). | Per-function probe cases filled by the oracle, plus generated cases for each milestone. |
| Reported result | Function depth about 82% "estimated" (docs/function-parity.md "Depth") *(gridcraft's claim)*. | 423 of 506 in-scope functions have at least 99.9% of their cells within 15 digits. Calipers: 99.88% of cells bit-exact, and 160 of 173 workbooks have every cell within 15 digits. |

A 1e-9 tolerance passes values that differ in the 10th to 15th significant digit. Most of
RustSheet's remaining gap is numeric: 99,419 visibly-off numbers against 850 wrong types, errors or
texts (docs/progress.md). A 1e-9 check would hide the part of that gap that lies in the 10th to 15th
digit, though not the numbers that are further off.

## Function coverage
gridcraft's function names were taken from the source: every `f!`, `base_fn!` and `im1!` spec in
`crates/functions/src`, plus `SPECIAL_FUNCTIONS` in `crates/calc/src/eval/special.rs`, without the
test-only `ADD2`. That gives 514 names: 513 from eval/catalog/excel-functions.tsv, plus USDOLLAR. This
agrees with the README's 513. The total row of the "Coverage" table in docs/function-parity.md (505) is out of date,
and several rows of that doc and of docs/file-format-parity.md appear more than once.

Against RustSheet's catalog (527 names, 506 in scope, eval/catalog/scope.json):
- gridcraft lacks 2 in-scope functions, ECMA.CEILING and GETPIVOTDATA. RustSheet has both. This
  comes from the source scan: gridcraft's own list of missing functions names GETPIVOTDATA but not
  ECMA.CEILING, which is not on Microsoft's list.
- gridcraft has 9 functions RustSheet treats as out of scope: the 7 CUBE functions (they return
  `#N/A`), WEBSERVICE (it returns `#VALUE!`) and EUROCONVERT.
- gridcraft also has USDOLLAR, which is not on Microsoft's list. T-3119 probes whether Excel
  evaluates it.

## Architecture

| Topic | gridcraft | RustSheet |
|---|---|---|
| Recalculation | A dependency graph with range nodes. Range dependents sit in a per-column index of row intervals (sorted intervals, a segment tree of the largest end, a small pending set that is merged in batches), and ranges wider than 64 columns go to a per-sheet index (`crates/calc/src/recalc/graph.rs`). Recalc is incremental and single-threaded. | Every `evaluate()` recalculates everything. The graph is SPEC-0002 (T-0401), which waits for Phase 2 (decision 0027). Large lookup and criteria ranges are no longer rescanned for every formula (T-0448, T-0449). |
| Storage | Copy-on-write bands of 64 rows behind `Arc`, so undo snapshots are cheap (docs/architecture.md). | Chunks of 1,024 rows. Copy-on-write chunks are the M4 target (docs/architecture/overview.md). |
| Resource limits | 16 million cells per reference, nesting depth 400, LAMBDA calls bounded by the same cell budget (`crates/calc/src/eval.rs`). | A formula work budget (SPEC-0024), a recalculation work budget (SPEC-0028) and the stack budget for LAMBDA calls (decision 0080). |
| Numeric kernels | Lanczos gamma (g = 7, 9 terms), and Acklam's inverse normal followed by one Halley step (`crates/functions/src/special.rs`). | Fitted to Excel's bits where the evidence allows. In the tails Excel's NORM.S.INV is Wichura's AS 241, bit-exact on 611 of 616 probed cells. Its central region fits no candidate tried so far, including one Newton or Halley step from PPND16 (docs/excel-semantics/statistics.md, "NORM.S.INV is AS 241 PPND16 in the tails", T-1746). |

## gridcraft's Excel claims against RustSheet's evidence
gridcraft's docs describe Excel behaviours as observed. Where RustSheet already has oracle evidence:
- **They agree.**
  - INDEX with one index on a reference of several rows and columns is `#REF!` (probes/index-single-index, T-0213).
  - MATCH, VLOOKUP and HLOOKUP compare doubles exactly, while `=` and the criteria functions use 15 digits (excel-semantics/lookup.md, "Approximate-match lookups"; criteria.md, "Numeric `<`, `<=`, `>`, `>=` criteria compare the 15-digit value of the cell"). gridcraft also claims
    this for XLOOKUP, XMATCH and LOOKUP, which RustSheet has not probed yet (T-3106 probes them).
  - MIRR with only negative flows is -1 (financial.md, "MIRR error and edge cases").
  - The Analysis ToolPak functions give `#VALUE!` for a multi-cell reference in a value argument (`lift: arrays`, SPEC-0011).
  - Excel nests LAMBDA calls deeper than gridcraft's limit of 140. RustSheet's probe evaluates 300 levels (T-0444).
- **They disagree.** gridcraft's docs/function-parity.md (2026-10-15 row) says ADDRESS leaves a
  sheet_text of `TRUE` or `FALSE` unquoted. RustSheet's oracle evidence shows Excel quotes it:
  `ADDRESS(9,26,3,TRUE,"TRUE")` = `'TRUE'!$Z9` (lookup.md, "ADDRESS: argument errors first", T-2274).
  RustSheet keeps Excel's behaviour.
- **Leads without RustSheet evidence**, filed as probes in **T-3119**:
  - number text with a currency sign outside the parentheses (`$(1,234)`) or a percent sign after them (`(5)%`);
  - bare `"<="` and `">="` criteria over empty text;
  - USDOLLAR.

## Ideas, and what became of each
1. **An interval index for range dependents.** Adopted as an input: T-0401's Context now points here,
   so SPEC-0002 weighs this design next to the others it lists. Nothing changes before Phase 2.
2. **Checking against the cached values of any saved file.** Already done: calipers and corpus-bench
   are scored from their cached values, bit for bit rather than at 1e-9.
3. **The clean-room rule against committing Excel-produced files.** Not adopted. RustSheet's
   evidence is Excel-computed values and Excel-authored workbooks (directive 1).
4. **gridcraft's numeric kernels.** Not adopted. They are textbook approximations that are not
   fitted to Excel's bits. In the NORM.S.INV tails Excel uses AS 241, not Acklam's rational form. Porting any of
   its code would also need a licence and provenance review, which is the owner's call.
5. **Its Excel behaviour claims.** Used as leads only: T-3119.

## Left to the owner
A measured head-to-head, like the mog one, needs gridcraft built and run: through `gridcraft-cli` on
RustSheet's suites, and RustSheet on gridcraft's corpus if that is public. That means running
third-party code, which an issue-origin task may not do (decision 0158). If the owner wants it, the
method in mog-head-to-head-2026-10-22.md applies unchanged: a temporary engine adapter in
rustsheet-eval that is not merged, the closeness buckets of `closeness.rs`, and wall time and peak RSS
per workbook.
