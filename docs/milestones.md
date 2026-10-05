# Milestones

Each milestone has a **scope**, **exit criteria** (measurable; checked by agents and recorded in
`docs/status.md`), and the **eval suites** that measure it. A milestone's functions only join the
CI gate for synthetic suites once they are in scope. Later milestones can start in parallel
lanes when they don't depend on earlier ones, but exit criteria are checked in order.

Scoring: *bit-exact cell rate* = cells identical to Excel ÷ cells checked (strict, see
`architecture/eval-harness.md`).

---

## M0 — Scaffolding, harness, oracle, autonomous loop

**Scope:**
- Fork IronCalc and rename it.
- Eval harness: suites, strict scoring, gap report, known-failures ratchet.
- Excel oracle client, spec→xlsx builder, P0 generator.
- CI, quality ratchets, agent process (CLAUDE.md, reviewers, skills, task board), docs and plans.

**Exit criteria:**
- [x] `make check` and CI green on `main`.
- [x] `eval-check` gate active, with ironcalc-calc baseline 69,478 / 71,387 bit-exact (97.33%).
- [ ] Owner completes oracle setup. `rustsheet-oracle fill` produces sidecars for `smoke`,
  `probes` and `gen-m2` from a real Excel. Calibration: Excel-for-web vs cached values agree on
  ≥ 99.9% of ironcalc-calc cells (T-0004).
- [ ] One full autonomous work-loop iteration merged without human help (T-0010).
- [ ] Scheduled worker routines running (2 lanes).

## M1 — Core calculation semantics (no functions beyond trivial)

**Scope:**
- Literals: number, text, bool, error, blank, and the empty string.
- Operators and precedence: unary minus vs `^`, `%`, `&`, comparisons, the range/union/intersection operators.
- Coercion rules: text↔number↔bool in operators.
- Error propagation and precedence.
- Excel floating point: 15-digit display rules, near-zero snapping, number-literal parsing.
- References: A1, `$A$1`, mixed, ranges, whole rows and columns, cross-sheet, quoted sheet names, R1C1 parse/print.
- Recalculation correctness on chains.
- 1900 date serial basics, including the leap-year bug.
- **API facade v0** (spec first): `Workbook`, `SheetId`, `CellRef`, typed `Value`/`ErrorKind`,
  `edit()` → `ChangeSet` (computed by diff at first), `eval()`, plus Python binding on the facade.
  Remove the `Model<'a>` lifetime (this fixes the leak in the Python binding).

**Exit criteria:**
- `smoke`, `probes` and `gen-m1` (≥ 3,000 oracle cells, operators and references only) at 100% bit-exact.
  Measured by eval/baseline/gen-m1.txt reaching zero failing cells (T-0109: 978 left, in T-0105,
  T-0107, T-0130–T-0132, T-0134 and T-0221; T-0221 is an M2 task that also blocks M1).
- Known parity bugs from the IronCalc assessment fixed: `ROUND(1.005,2)`, `CONCAT(0.1+0.2)`,
  `DATE(1900,2,29)`, and the parser accepting trailing garbage.
- API v0 spec merged and implemented. Python `rustsheet.Workbook` passes binding tests.

## M2 — Top functions (the everyday spreadsheet)

**Scope** (in order of real-world frequency):
- **Aggregates and logic:** SUM, IF, AVERAGE, COUNT, COUNTA, MIN, MAX, ROUND (with ROUNDUP and
  ROUNDDOWN), IFERROR, AND, OR, NOT, ABS.
- **Lookup and conditional aggregates:** VLOOKUP, HLOOKUP, INDEX, MATCH, SUMIF, COUNTIF,
  SUMIFS, COUNTIFS.
- **Text:** CONCATENATE/CONCAT, LEN, LEFT, RIGHT, MID, TRIM, UPPER, LOWER.
- **Dates:** TODAY/NOW (deterministic clock injection), YEAR, MONTH, DAY.
- **Criteria and wildcard grammar** shared by the *IF(S) family.

**Exit criteria:**
- `gen-m2` (≥ 10,000 oracle cells across those functions, with edge pools) at 100%.
- ironcalc-calc cells using only M1/M2 functions at 100%.
- One implementation of aggregate-argument iteration and criteria matching, used by every
  consumer. Duplication baseline reduced by ≥ 15%.

## M3 — Dependency graph, incremental and lazy recalculation

**Scope (spec first):**
- Persistent dependency graph: cells, ranges via an interval index, names.
- Dirty propagation; volatile set; dynamic dependencies (INDIRECT/OFFSET).
- `CalcMode` Automatic / Manual / OnDemand (compute-on-read); `watch`.
- Excel circular-reference semantics, with iterative calc settings.
- `ChangeSet` derived from propagation instead of diffing.
- `precedents`/`dependents`.
- Benchmark suite and perf CI gate.

**Exit criteria:**
- Metamorphic suite: incremental == full recalc on 10k random edit sequences.
- 1-cell edit on a 1M-formula chain model recalculates in < 5 ms when only O(1) dependents change.
- Full recalc throughput no worse than fork baseline.
- All eval suites unchanged (no regressions).

## M4 — Storage, memory and structural edits

**Scope:**
- Chunked columnar copy-on-write storage.
- Interned strings (single copy) and deduplicated styles/formats.
- Formula families stored once.
- `Snapshot` (O(1)) and `Scenario`.
- Insert/delete rows/cols/cells and move ranges via `RefMap`, with Excel-exact reference
  adjustment (#REF!, range expansion rules) and no formula re-parse.

**Exit criteria:**
- ≤ 64 B per numeric cell and ≤ 100 B per text cell (repeated text shared) on benchmark models.
- Insert 1 row in a 1M-cell sheet in < 50 ms.
- Structural-edit oracle suite (edit scripts replayed in Excel via COM/Graph) at 100%.

## M5 — Function families I: math, logical, text, information

**Scope:** every function in these Excel categories, including SUMPRODUCT, MOD/QUOTIENT edge
cases, ROUND family, TEXT (with number-format subset), VALUE, FIND/SEARCH, SUBSTITUTE, REPT,
the IS* functions, TYPE, N, ERROR.TYPE, SWITCH, IFS, XOR, CHOOSE.

**Exit criteria:** catalog coverage 100% for these categories; bit-exact ≥ 99.9% on generated
suites (≥ 20k cells); 100% on ironcalc-calc for these categories.

## M6 — Lookup & reference, date & time

**Scope:**
- **Lookup:** XLOOKUP, XMATCH (all match/search modes), LOOKUP, OFFSET, INDIRECT (A1/R1C1),
  ADDRESS, ROW(S)/COLUMN(S), CHOOSECOLS/ROWS.
- **Date and time:** DATEVALUE/TIMEVALUE parsing (en-US), EDATE/EOMONTH, WORKDAY/NETWORKDAYS(.INTL),
  DATEDIF, YEARFRAC (all bases), WEEKNUM/ISOWEEKNUM, DAYS360.

**Exit criteria:** same as M5 for these categories.

## M7 — Dynamic arrays & modern Excel

**Scope:**
- Spill semantics and `#SPILL!`; implicit intersection (`@`); array broadcasting rules.
- FILTER, SORT, SORTBY, UNIQUE, SEQUENCE, RANDARRAY (seeded).
- LET, LAMBDA, MAP/REDUCE/SCAN/MAKEARRAY/BYROW/BYCOL.
- TEXTSPLIT/TEXTBEFORE/TEXTAFTER, VSTACK/HSTACK/TAKE/DROP/WRAPROWS/TOCOL.
- GROUPBY/PIVOTBY, REGEX*.

**Exit criteria:** generated dynamic-array suite (≥ 10k cells, spill shapes included) at ≥ 99.9%;
spill extents identical to Excel.

## M8 — Names, tables, structured references, multi-sheet

**Scope:**
- Defined names (workbook and sheet scope, LAMBDA names).
- Tables and structured references (`Table1[@Col]`, `#Totals`, …).
- 3D references (`Sheet1:Sheet3!A1`).
- External workbook references (read cached values).
- Sheet add, rename and move with reference rewriting.

**Exit criteria:** oracle suites for each feature at 100%; POI corpus subset ≥ 99.5%.

## M9 — Numerical parity: statistical, financial, engineering, database

**Scope:**
- Distributions and inverses, regression (LINEST/LOGEST), forecasting.
- Financial: IRR/XIRR/RATE solvers with Excel-matching convergence and #NUM! behavior; bond
  functions.
- Engineering: Bessel, complex numbers, CONVERT, base conversions.
- Database functions D*.

**Exit criteria:** ironcalc-calc at ≥ 99.95% bit-exact overall (from 97.33%); dense oracle sweeps
per numeric function at ≥ 99.9% bit-exact and 100% within 4 ULP.

## M10 — Formatting & display

**Scope:**
- Number-format engine (all built-in and custom formats, conditional sections, dates,
  fractions, scientific).
- `formatted()` cell text identical to Excel's `.Text`.
- TEXT/FIXED/DOLLAR.
- Style storage and round-trip (deduplicated).
- Locale plumbing (en-US first).

**Exit criteria:** formatted-text oracle suite (≥ 10k cells) at 100%.

## M11 — Fidelity and performance at scale

**Scope:**
- Real-world corpora: POI, FUSE/SpreadsheetBench via xlsx-corpus-bench, Enron (performance).
- xlsx round-trip preservation of unknown parts.
- Data tables (TABLE()).
- Iterative calculation parity.
- Parallel recalc.
- Comparison against Excel desktop timings, LibreOffice and Formualizer.

**Exit criteria:**
- Formula-cell match on xlsx-corpus-bench ≥ 99.8%, matching or beating the best published engine.
- Full recalc faster than Excel desktop on the benchmark suite.

## M12 — API v1, bindings & release

**Scope:**
- API v1 freeze (semver-checked).
- Full Python API: bulk/numpy, GIL release, `Calculator`.
- Java bindings via the FFI facade (UniFFI or Panama).
- Documentation site; fuzzing (cargo-fuzz on parser and xlsx importer); release automation.

**Exit criteria:** API v1 tagged; Python and Java packages published from CI; fuzzers run 24 h
with no crash.
