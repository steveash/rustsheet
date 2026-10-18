# Milestones

Each milestone has a **scope**, **exit criteria** (measurable; checked by agents and recorded in
`docs/status.md`), and the **eval suites** that measure it. A milestone's functions only join the
CI gate for synthetic suites once they are in scope. Later milestones can start in parallel
lanes when they don't depend on earlier ones, but exit criteria are checked in order.

## Phase order (decision 0027)

Milestones are worked in this order, not in numeric order. The *current milestone* (used by
the planner, the gap-triager and the steward's critical lanes) is the first one in this order
whose exit criteria are not met.

| Phase | Milestones, in order | Status |
|---|---|---|
| 1. Parity | M0, M1, M2, M5, M6, M7, M8, M9, M10, M11 | active |
| 2. Architecture and performance | M3, M4, M13 | deferred until Phase 1 exits |
| 3. Release | M12 | after Phase 2 |
| 4. Desktop oracle | M14 | after Phase 3; not started (decision 0039) |

**Phase 1 exit:** every Excel function is in the catalog with its parity scope (T-0040). Every
in-scope function has oracle-backed cases covering each parameter mode and edge pool, and
reaches ≥ 99.9% bit-exact and 100% within 4 ULP. Each remaining failure has a filed task or a
`DEVIATION` entry. "Bit-exact" percentages here and in every milestone count only scored cells
not listed as accepted deviations, which are scored in their own column (decision 0036). Accepted
deviations are directive-5 limits only (decision 0052). The exit criteria of M1, M2, M5–M10 and M11 are met.

A failure whose only open question needs desktop Excel evidence has its filed task in M14
(Phase 4). It never blocks the exit of Phase 1, any other phase or any milestone. excel-graph is
the oracle of record for Phases 1–3 (decision 0039).

Out of scope: functions whose result depends on systems outside the workbook (web and online
services, external data servers, code and add-ins; list in decision 0027).

When all of this holds, the steward adds the line `**Phase 1:** exited <YYYY-MM-DD>` to
docs/status.md. Until then, the M3, M4 and M13 tasks stay `blocked`. Parity behavior that
needs the Phase 2 dependency graph (circular references and iterative calculation, dynamic
dependencies, TABLE()) still gets its oracle cases in Phase 1; only its implementation waits.

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
- [x] Owner completes oracle setup. `rustsheet-oracle fill` produces sidecars for `smoke`,
  `probes` and `gen-m2` from a real Excel. Calibration: Excel-for-web vs cached values agree on
  ≥ 99.9% of ironcalc-calc cells (T-0004: 99.96% of 71,765 cells, docs/research/oracle-calibration.md).
- [x] One full autonomous work-loop iteration merged without human help (#3, T-0200, 2026-10-02; T-0058).
- [x] Scheduled worker routines running (2 lanes): the sonnet and opus workers and the daily steward
  in docs/process/routines.md (T-0058).

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
- `smoke`, `gen-m1` (≥ 3,000 oracle cells, operators and references only) and the frozen M1
  probe list at 100% bit-exact, excluding cells listed as accepted deviations (decision 0036).
  Measured for gen-m1 by eval/baseline/gen-m1.txt reaching zero failing cells (T-0109: 978 left,
  in T-0105, T-0107, T-0130–T-0132, T-0134 and T-0221; 8 left on 2026-10-10: 6 in T-1302, the
  integer-exponent power rule, and 2 in T-0221, an M2 task that also blocks M1; 2 left on
  2026-10-12, both negative non-integer exponents, split from T-0221 into M1 task T-1421; 0 left on 2026-10-13 after T-1319, #640).
- The frozen M1 probe list is `eval/milestones/m1-probes.txt` (decision 0043): 89 `probes` cases
  in M1 scope, fixed at commit e5c7ae9. Its cases have no F line left in
  eval/baseline/probes.txt (61 at the freeze; `make eval` reports it once T-1146 lands). Probe
  cases outside the list, including every probe added after the freeze, count toward the milestone
  decision 0051 assigns them and never toward M1, whatever their tag.
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
- Every `probes` case assigned M0, M1 or M2 at 100% bit-exact, excluding accepted deviations
  (decision 0036). Decision 0051 assigns each case one milestone. The frozen M1 list is M1. A
  case listed in `eval/milestones/probe-milestones.tsv` takes the milestone given there. Each
  `probes/catalog/*` cell takes the milestone of the function it calls. Any other case takes its
  case.json milestone tag, or M2 if it has none. Probes of later-milestone functions and
  features count toward their own milestone, not M2 (the owner's answer to decision 0043). At
  9ad32ae1, 74 of the 2,881 F lines in eval/baseline/probes.txt were assigned M0–M2.
- ironcalc-calc cells using only M1/M2 functions at 100%.
- One implementation of aggregate-argument iteration and criteria matching, used by every
  consumer. Duplication baseline reduced by ≥ 15%.

## M3 — Dependency graph, incremental and lazy recalculation (Phase 2)

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

## M4 — Storage, memory and structural edits (Phase 2)

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
suites (≥ 20k cells); 100% on ironcalc-calc for these categories;
its `probes` cases (decision 0051) at 100% bit-exact, excluding accepted deviations.

## M6 — Lookup & reference, date & time

**Scope:**
- **Lookup:** XLOOKUP, XMATCH (all match/search modes), LOOKUP, OFFSET, INDIRECT (A1/R1C1),
  ADDRESS, ROW(S)/COLUMN(S), CHOOSECOLS/ROWS.
- **Date and time:** DATEVALUE/TIMEVALUE parsing (en-US), EDATE/EOMONTH, WORKDAY/NETWORKDAYS(.INTL),
  DATEDIF, YEARFRAC (all bases), WEEKNUM/ISOWEEKNUM, DAYS360.

**Exit criteria:** same as M5 for these categories, and
its `probes` cases (decision 0051) at 100% bit-exact, excluding accepted deviations.

## M7 — Dynamic arrays & modern Excel

**Scope:**
- Spill semantics and `#SPILL!`; implicit intersection (`@`); array broadcasting rules.
- FILTER, SORT, SORTBY, UNIQUE, SEQUENCE, RANDARRAY (seeded).
- LET, LAMBDA, MAP/REDUCE/SCAN/MAKEARRAY/BYROW/BYCOL.
- TEXTSPLIT/TEXTBEFORE/TEXTAFTER, VSTACK/HSTACK/TAKE/DROP/WRAPROWS/TOCOL.
- GROUPBY/PIVOTBY, REGEX*.

**Exit criteria:** generated dynamic-array suite (≥ 10k cells, spill shapes included) at ≥ 99.9%;
spill extents identical to Excel;
its `probes` cases (decision 0051) at 100% bit-exact, excluding accepted deviations.

## M8 — Names, tables, structured references, multi-sheet

**Scope:**
- Defined names (workbook and sheet scope, LAMBDA names).
- Tables and structured references (`Table1[@Col]`, `#Totals`, …).
- 3D references (`Sheet1:Sheet3!A1`).
- External workbook references (read cached values).
- Sheet add, rename and move with reference rewriting.

**Exit criteria:** oracle suites for each feature at 100%; POI corpus subset ≥ 99.5%;
its `probes` cases (decision 0051) at 100% bit-exact, excluding accepted deviations.

## M9 — Numerical parity: statistical, financial, engineering, database

**Scope:**
- Distributions and inverses, regression (LINEST/LOGEST), forecasting.
- Financial: IRR/XIRR/RATE solvers with Excel-matching convergence and #NUM! behavior; bond
  functions.
- Engineering: Bessel, complex numbers, CONVERT, base conversions.
- Database functions D*.

**Exit criteria:** ironcalc-calc at ≥ 99.95% bit-exact overall (from 97.33%); dense oracle sweeps
per numeric function at ≥ 99.9% bit-exact and 100% within 4 ULP;
its `probes` cases (decision 0051) at 100% bit-exact, excluding accepted deviations.

## M10 — Formatting & display

**Scope:**
- Number-format engine (all built-in and custom formats, conditional sections, dates,
  fractions, scientific).
- `formatted()` cell text identical to Excel's `.Text`.
- TEXT/FIXED/DOLLAR.
- Style storage and round-trip (deduplicated).
- Locale plumbing (en-US first).

**Exit criteria:** formatted-text oracle suite (≥ 10k cells) at 100%;
its `probes` cases (decision 0051) at 100% bit-exact, excluding accepted deviations.

## M11 — Real-world fidelity

**Scope:**
- Real-world corpora: POI, FUSE/SpreadsheetBench via xlsx-corpus-bench.
- xlsx round-trip preservation of unknown parts.
- Data tables (TABLE()) and iterative calculation parity. The oracle cases come in Phase 1;
  the implementation needs the M3 graph.

**Exit criteria:** formula-cell match on xlsx-corpus-bench ≥ 99.8%, matching or beating the best
published engine;
its `probes` cases (decision 0051) at 100% bit-exact, excluding accepted deviations.

## M12 — API v1, bindings & release

**Scope:**
- API v1 freeze (semver-checked).
- Full Python API: bulk/numpy, GIL release, `Calculator`.
- Java bindings via the FFI facade (UniFFI or Panama).
- Documentation site; fuzzing (cargo-fuzz on parser and xlsx importer); release automation.

**Exit criteria:** API v1 tagged; Python and Java packages published from CI; fuzzers run 24 h
with no crash.

## M13 — Performance at scale (Phase 2)

Split out of M11 by decision 0027.

**Scope:**
- Enron/VEnron performance models.
- Parallel recalc.
- Comparison against Excel desktop timings, LibreOffice and Formualizer.

**Exit criteria:** full recalc faster than Excel desktop on the benchmark suite.

## M14 — Desktop-oracle parity (Phase 4)

Added by decision 0039. Planned after Release; not started. It starts when the owner sets up the
`excel-desktop` backend: a Windows machine with Microsoft 365 Apps and a self-hosted runner, as in
docs/architecture/oracle.md, "Owner setup — Windows desktop backend". Until
`rustsheet-oracle status` shows `excel-desktop OK`, its tasks stay `blocked`. It does not gate
Phases 1–3.

**Scope:** what only desktop Excel can answer:
- Desktop-only behavior: `CELL("width")`, and workbooks Excel for the web refuses or cannot open.
- Conditional-format evaluation (T-1051).
- Typed cell input (T-1016) and fill-handle behavior (T-0171).
- Structural-edit semantics that a static workbook cannot probe (T-0198).
- Web-vs-desktop conflicts and desktop confirmation of rules implemented on excel-graph evidence in
  Phases 1–3 (T-1075).

**Exit criteria:**
- The excel-desktop backend is calibrated against excel-graph and the cached values of
  ironcalc-calc (T-1074).
- Every CONFLICTING entry in docs/excel-semantics/ is settled by desktop evidence. Each
  web-evidenced rule listed in T-1075 is confirmed, or it has a follow-up task.
- The M14 tasks are done, or dropped with a reason.
