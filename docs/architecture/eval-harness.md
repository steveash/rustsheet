# Eval harness (normative)

`crates/rustsheet-eval` measures Excel compatibility. It is engine-agnostic (trait `Engine` in
`engine.rs`), so other engines, such as upstream IronCalc or a rewritten core, can be scored
side by side.

## Suites
`eval/suites/<name>/suite.toml`:

| key | meaning |
|---|---|
| `name`, `description`, `license`, `origin` | identity and provenance (license decides vendor vs fetch) |
| `kind` | `xlsx` (workbooks) or `spec` (`*.case.json`) |
| `root`, `include`, `exclude` | where the cases are (globs relative to `root`) |
| `use_cached_values` | for xlsx: Excel's cached `<v>` values are the oracle (when no sidecar exists) |
| `require_excel_authored` | only trust caches written by Microsoft Excel (`docProps/app.xml`; XlsxWriter's fake `12.0000` rejected) |
| `oracle_dir` | optional, relative path inside the suite directory: sidecars live there, mirroring the case path, instead of next to the case (for fetched suites, whose cases sit in the git-ignored cache; decision 0019) |
| `cached_overrides` | xlsx: optional file (relative to the manifest) replacing single cached values that are not en-US evidence with cells of committed `excel-graph`/`excel-desktop` sidecars (`overrides.rs`, decision 0032); an excel-graph evidence cell whose probe companion `(x)=0` is FALSE also carries decision 0042's kept-subnormal rule (decision 0090) |
| `gate` | part of the CI no-regression gate |
| `baseline` | `cells` (default), `cases` (large corpora) or `none` |
| `milestone` | which milestone(s) the suite measures |
| `[fetch]` | optional pinned external source: `url` (`https://` or `file://`), exactly one pin (`rev` = 40-hex git commit, or `sha256` of a `.zip`/`.tar*` archive), optional `subdir` and `include` globs |

External suites (`[fetch]`) live in `eval/.cache/suites/<name>` (gitignored) after `rustsheet-eval fetch`;
their `root` is that directory. A suite whose cache is missing or stale (marker `.rustsheet-fetched`
differs from the pin) is listed as `not fetched` and skipped, with a note, by `run`/`check`/`update-baseline`.
Use this for sources that are too large or not redistributable (GPL / CC-BY-SA); never vendor them.

Current suites:
- `ironcalc-calc`: 207 Excel-authored workbooks, 71k cells.
- `poi`: Apache POI Excel-authored xlsx/xlsm (external, fetched with `rustsheet-eval fetch --suite poi`; 3.5k cells).
- `smoke`: hand-written core semantics.
- `probes`: Excel-behavior questions awaiting the oracle.
- `gen-m2`: generator output (30 cases, 12,000 cells), oracle-filled and gated (T-0203).

## Case formats
**`rustsheet-case/v1`** (`*.case.json`):
```json
{"format":"rustsheet-case/v1","description":"...","tags":["M1"],
 "sheets":[{"name":"Sheet1","cells":{"A1":1,"A2":"text","A3":"=A1+1","A4":true,"A5":{"t":"e","v":"#N/A"}}}],
 "expected":{"Sheet1!A3":{"t":"n","v":2}}, "expected_source":"hand", "check":["Sheet1!A3"]}
```
Input conventions:
- JSON number is a number and JSON bool is a boolean. A number reaches Excel and the engine as the exact
  double the case holds: spec2xlsx writes a number that needs 17 significant digits with the shortest
  text that parses back to it (XlsxWriter alone would round to `{x:.16G}`; T-0289).
- A string starting with `=` is a formula, written with **dynamic-array (M365) semantics**.
- Any other string is text. Use the tagged form for text that starts with `=` or for errors.
- The empty string `""` is materialized as `=""`.
- `{"entered":"(1,000)"}` is text entered *as a user types it* (T-0280): Excel's input parser picks
  the value and number format. spec2xlsx leaves the cell blank and lists it in docProps custom
  properties `rustsheet.entered.<n>`; the excel-graph backend writes it with a range `values`
  PATCH before calculating (and logs the `numberFormat` Excel chose as `ENTERED ...` on stderr),
  excel-desktop sets `Range.Value` (that this parses exactly like typing is UNVERIFIED: no desktop run yet), libreoffice refuses. The engine adapter enters it as
  `Input::Parse`. Formulas can only observe the value: `CELL("format")` is `#VALUE!` on excel-graph.
- `{"entered_r1c1":"=R1C2+1"}` is typed the same way in R1C1 mode (T-3113, decision 0179): recorded with
  a fourth element `"r1c1"` and written with a range `formulasR1C1` PATCH (excel-desktop:
  `Range.FormulaR1C1`, UNVERIFIED like `Range.Value`). The engine adapter converts
  it to A1 text relative to its cell (`rustsheet_engine::r1c1_formula_to_a1`) and enters that as
  `Input::Parse`. Typed text Excel for the web refuses (PATCH 400 `InvalidArgument`) leaves the cell blank, and so
  does R1C1 text the engine cannot parse; probe a refusal through a dependant such as `=ISBLANK(A1)`.
- Optional per-sheet `"legacy_formulas": ["H1"]` lists formula cells that spec2xlsx writes as plain
  legacy `<f>` formulas (no dynamic-array `cm`), to probe what Excel shows for a non-array formula
  (T-1077). The engine adapter enters them as `Input::LegacyFormula`, which adds the implicit
  intersections xlsx import adds to a plain `<f>` (`=A1:A2` reads as `=@A1:A2`; T-1096).
- Optional top-level `"invariant": ["Sheet1!A1"]` lists run-invariant volatile cells to score (see Scoring).
- Optional per-sheet `"hidden_rows": [2, 5]` (1-based rows written `hidden="1"`) and
  `"tables": [{"name": "Data", "ref": "A1:B9", "filter": true}]` (T-1036). A table's header row is
  the first row of `ref` and must hold literal text. With `"filter": true` spec2xlsx gives the table's
  autoFilter a values filter on its first column that keeps exactly the visible data rows (those
  cells must be literal text or whole numbers, and no value may be both kept and hidden), so Excel
  reads the hidden rows as filtered. The engine adapter loads both through `Txn::add_table` and
  `Txn::set_row_hidden`. With `"totals_row": true` the last row of `ref` is the table's totals row; its
  cells must be labels or formulas, which spec2xlsx writes with the totals-row metadata Excel needs. The
  facade cannot add a totals row yet, so the adapter registers the table without that row (T-1671).
  Formulas use structured references as typed (`Sales[@Qty]`, a bare `Sales`); spec2xlsx stores their
  file form (`Sales[[#This Row],[Qty]]`, `Sales[]`; T-1607). References must name their table.
- Optional top-level `"date_system": 1904` (default 1900) writes `<workbookPr date1904="1"/>`, and
  optional per-sheet `"number_formats": {"B1": "[h]:mm"}` gives cells (blank ones included) a number
  format (T-1268; an `entered` cell cannot have one, Excel picks its format). The engine adapter
  creates the workbook with `WorkbookOptions::date_system` and applies formats with
  `Txn::set_number_format`. Number formats change only what a cell displays, which excel-graph
  cannot report (it reads cached values), so they are for cases that formulas read through.
- Optional `"names": {"rate": "=Sheet1!$B$1"}` declares workbook-scoped defined names (any
  formula Excel accepts as a name body, e.g. an intersection). A key `Sheet1!name` is a name scoped
  to that sheet.
- Optional `"sheet_ops"` lists sheet edits made after the workbook loads and before it is
  recalculated (T-1608): `{"op":"add","sheet":"New"}` appends a sheet, `{"op":"rename",
  "sheet":"Data","to":"Inputs"}` renames one (formulas that refer to it are rewritten), and
  `{"op":"move","sheet":"Data","to":0}` moves one to a 0-based position, and
  `{"op":"insert_row","sheet":"S","row":1}` inserts one blank row above the 1-based row, shifting
  the cells below down (T-1933; Graph `range(address='1:1')/insert`). T-2323 adds `delete_row`
  (`row`), `insert_column` and `delete_column` (`column`, a letter), `delete_sheet` and
  `convert_table` / `delete_table` (`table`); see docs/architecture/oracle.md for the Graph calls
  and decision 0098 for the engine side (`convert_table` is refused there). Any other op or key is
  refused. spec2xlsx records them as docProps custom properties `rustsheet.sheetop.<n>`. The
  excel-graph backend replays them in the session after the typed entries and keeps its canary
  sheet last: an added sheet goes just before the canary, and a move may not reach the canary's
  position. The other backends refuse such cases. The engine adapter applies each op as its own
  `Workbook::edit` (`Txn::add_sheet`, `rename_sheet`, `move_sheet`), and a row insert through the
  harness-only `Workbook::insert_rows` (cargo feature `harness_row_insert`, decision 0076) and a
  recalculation. Keys in `expected` and `check` use the sheet names and cell addresses after the
  edits; the case's own inputs stay at their written addresses, so a row-insert case lists its
  scored cells in `check`.

The same conventions are implemented by `tools/.../spec2xlsx.py` (for Excel) and by
`engine.rs::evaluate_spec` (for RustSheet).

**Values** (shared by expectations, oracle files and reports):
- `{"t":"n","v":1.5}` number
- `{"t":"s","v":"x"}` text
- `{"t":"b","v":true}` boolean
- `{"t":"e","v":"#DIV/0!"}` error (add `"stored":true` for a stored `t="e"` constant instead of typed input: spec2xlsx writes it so, T-0999, and the engine adapter enters it as `Input::Value` with the stored-name kind, `#ERROR!` if unknown, as xlsx import does, T-1221)
- `{"t":"empty"}` blank

JSON numbers are written shortest-round-trip, so they are exact.

**`rustsheet-oracle/v1`** sidecar (`<case>.oracle.json`, next to the case or under `oracle_dir`):
`{format, source{kind, application, version, produced_at, input_sha256, workbook_name?}, sheets{Sheet:{A1:value}}}`.
`workbook_name` (since T-2042) is the file name the workbook had in the oracle (`<sha256>.xlsx` for excel-graph); the adapter gives the engine the same name (`WorkbookOptions::name`), so `CELL("address")` of a reference on another sheet compares.
`input_sha256` hashes the `spec2xlsx` output, which is deterministic since T-0015 (creation time pinned in `docProps/core.xml`). Sidecars filled before T-0015 carry hashes of non-reproducible bytes: treat them as legacy; they refresh on the next fill, and values do not depend on the hash.
Oracle values override hand expectations. `libreoffice` results are never written as sidecars.
A sidecar may also hold `text{Sheet:{A1:"shown text"}}`: Excel's displayed text of the cells a spec
case lists in `"text": ["Sheet!A1", ...]` (T-1610, decision 0068). spec2xlsx records those cells in
the `rustsheet.text.<n>` custom properties, and the excel-graph backend reads each one's
`Range.text` after the recalculation, in the same session.
Tools never re-derive case discovery: `rustsheet-eval cases <suite>` prints each case's path,
sidecar path, formula-cell count and whether the suite trusts its cached values.

## Scoring
Every formula cell (including array-formula members) that has an expectation is checked.
Formulas calling NOW/TODAY/RAND/RANDBETWEEN/RANDARRAY are skipped (counted as `skipped_volatile`),
and so is every formula that reads one through a static reference or a defined name. A spec case
opts cells back in with `"invariant": ["Sheet!A1", ...]` (T-1283): cells whose formula calls a
volatile function, or reads a volatile cell, but whose value cannot change between runs
(`RANDBETWEEN(5,5)`, `ROWS(RANDARRAY(3,2))`, `ISNUMBER(NOW())`). A listed cell is scored, and taint
does not flow through it to its dependents. Rules:
- every listed cell must be a volatile formula cell, otherwise the case fails to load (a stale listing);
- a listed value must not depend on the actual time either, because the engine adapter runs on a
  fixed mock clock (`NOW()>45000` is not invariant here);
- `rustsheet-oracle fill` recalculates a case with `invariant` cells a second time without the
  cache and leaves pending (prints `VARIES ...`) every listed cell whose two values differ.

A formula that calls an add-in function (`_xll.` prefix, outside string literals;
`formula_calls::calls_add_in`) is not scored either, and is counted as `skipped_add_in` (T-0462).
Decision 0027 puts add-ins out of scope, and Excel's cached value came from the add-in (calipers
pv26_003's S&P Capital IQ `_xll.ciqfunctions.udf.CIQHI` cells). Unlike volatile taint, only the
calling cell is left out: its dependents are scored, so a miss they show stays visible. report.md's
out-of-scope section lists each suite's count.

| Verdict | Meaning |
|---|---|
| `pass` | bit-identical (−0 == +0), same text/bool/error |
| `numeric_ulp` | ≤ 4 ULP apart (operation order / rounding) |
| `numeric_close` | relative error ≤ 1e-9 (algorithm/precision difference) |
| `numeric_mismatch`, `text_mismatch`, `bool_mismatch` | wrong value |
| `error_mismatch` | different error kinds |
| `unsupported` | engine returned `#NAME?`/`#N/IMPL` where Excel did not |
| `type_mismatch` | different value types |

Displayed text is scored apart from values (`text_expect.rs`): each listed cell with a sidecar
text passes when the engine's `Workbook::formatted` text is the same string. Text verdicts never
change a case's status, `checked`/`passed`, the baselines or the gate; the report's "Displayed text"
section shows each suite's text pass rate and every failing text cell.

Only `pass` counts as passing. Case statuses:
- `pass` / `fail`
- `pending`: no expectations yet
- `engine_error` / `panic` / `timeout` (60 s): a panic or timeout is always a bug
- `invalid_case`

## Outputs
- `eval/runs/latest/report.md` (git-ignored): suite table; cases that did not run; functions
  ranked by failing cells; failing examples per function; Excel evidence per function; out-of-scope
  functions. This is the hill-climbing to-do list. Function counts follow the catalog's `scope`
  (decision 0027), read from the generated `eval/catalog/scope.json` (decision 0040): `scope: out`
  functions leave the function tables and get their own section with their reason (and the per-suite
  count of unscored add-in cells, see Scoring); the evidence
  section lists in-scope catalog functions with no Excel evidence, the `oracle: none` ones that
  need another evidence source, and, by catalog category, the ones below the evidence floor
  (fewer than 20 Excel cells; T-0056, decision 0062). Suite cell counts and baselines still cover every checked cell.
  A "Catalog coverage per milestone" section (T-2112, `catalog_coverage.rs`) has one row per
  function-milestone report row (`REPORT_ROWS`, as in the function-scope section; M0 and M1 count in M2; not cumulative): its
  in-scope functions, how many have at least 20 Excel cells, how many have an Excel-backed scored
  cell in a generated suite tagged with that milestone, and how many have both. Functions missing
  either are listed by name, tagged `oracle: none` or `volatile`. Informational; never gated.
  A "Visibly off (fifteen digits)" section (T-0061, decision 0088) splits each suite's in-scope
  scored cells into the closeness buckets of `closeness.rs` (bit-exact, 1 ulp, 2–4 ulp,
  `digits15` within half a unit in the 15th significant digit, `visible_numeric`,
  `visible_other`) and lists the functions with visibly-off cells, worst first. A visibly-off
  cell is blamed on the functions it calls other than glue (`closeness::GLUE`: IF, SUM, DATE,
  INDEX and the like), or on its glue when it calls nothing else.
  A "Parity tiers (glue-aware)" section (T-0068, decision 0118, `attribution.rs`) counts the
  in-scope functions per worst-cell tier (`exact`, `ulp4`, `digits15`, `rate999`, `rate99`,
  `below99`, `unscored`) and how many of them use the fallback. Informational; never gated.
- `report.json`: the same data, machine-readable. Each function's `closeness` holds its bucket
  counts and `visible` its blamed visibly-off cells; each suite's `closeness` holds its in-scope
  bucket counts. Decision 0118's counts sit beside them (T-0068): each function's `excel` holds
  the non-deviation Excel-evidence cells that call it, `attributed` those whose only non-glue call
  it is (`closeness::sole_call`), `tier` the worst-cell tier and `attribution` whether `attributed`
  (`sole`) or, below 20 attributed cells, `excel` (`all`) placed it. Each function's
  `real_world_calls` counts the formula cells of the Excel-saved corpora that call it, the suites
  the top-level `real_world_suites` names (calipers and poi; T-0079). Each suite's `workbooks`
  counts its cases with a scored in-scope cell: `all_exact`, `all_within15` and `any_visible`.
  It uses the suite's `closeness` cells (hand-written expectations and deviation passes included),
  so it is suite scope, not Excel-only evidence.
  The top-level `tiers` holds the per-tier function counts of that section, over the catalog's
  in-scope set. `run` (T-0080, `run_meta.rs`) records the run itself: `commit` (`git rev-parse
  --short=8 HEAD`, or null when git fails), `date` (UTC, `YYYY-MM-DD`) and `skipped_suites`
  (`{name, reason}` for each suite under eval/suites that did not run: `not fetched` or `not
  selected`); report.md prints it under the title. `date` and `commit` change on every run, so no
  test or golden file compares a whole report.json.
  Each `argument_coverage[]` row's `unseen_pairs` (T-3277) lists the expected argument-class pairs
  (parameter names and `ArgClass` of each) that its `pairwise` share counts but no Excel-evidence call has passed.
- `eval/baseline/<suite>.txt`: known failures (see `docs/process/quality-gates.md`).
- `eval/baseline/visible-parity.json`: the visible-parity lock (decision 0088). Task PRs never lower
  it; the steward does, daily, with `make eval-lower-visible-lock` (decision 0093).

## Milestone probe lists
`eval/milestones/<name>.txt` (decisions 0043, 0044) freezes the cases a milestone's probe criterion
counts: `#` comment lines, then one case id per line in the form of the baseline files
(`probes/<path>.case.json`); `m1-probes.txt` is the only list today. `run` and `check` parse every
list (`crates/rustsheet-eval/src/milestone_lists.rs`) and fail, naming the file and line, on a
duplicate or on a case that no suite discovers. report.md has a "Milestone probe lists" section
(report.json: `milestone_lists`) with one row per list: cases, scored cells, bit-exact,
deviations, known failures (cells), and the failing case ids. Known failures are counted as the
baseline counts its F lines, so they equal the list's F lines in `eval/baseline/`. The lists are
an exit measure, not a gate on their pass rate; editing one needs a decision record
(`scripts/check_governance.py`).

## Probe milestones
Decision 0051 gives every `probes` case one milestone, and every checked cell of
`probes/catalog/*` its own (`crates/rustsheet-eval/src/probe_milestones.rs`). The first rule that
applies decides: (1) a case on `eval/milestones/m1-probes.txt` is M1; (2) a line of
`eval/milestones/probe-milestones.tsv` sets the milestone, either of a case
(`case<TAB>milestone<TAB>subject`) or of one of its cells (`case<TAB>Sheet!A1<TAB>milestone<TAB>subject`,
which beats a case line); (3) a catalog cell takes the highest `eval/catalog/scope.json`
milestone of the functions its formula calls, apart from the observing wrappers OR, IFERROR,
ERROR.TYPE and INDEX, and M2 when it calls none; (4) any other case takes its case.json
milestone tag (`M\d+`, the highest when there are several); (5) an untagged case is M2. `run`
and `check` parse the mapping file and fail, naming the line, on a malformed line, a milestone
that is not `M\d+`, a duplicate case or cell, or a case that no suite discovers. report.md has a
"Probe milestones (decision 0051)" section (report.json: `probe_milestones`) with one row each
for M0–M2 (combined), M5, M6, M7, M8, M9, M10 and M11, plus a row for any other milestone a case
takes: cases, scored cells, bit-exact, deviations and known failures (cells). Known failures are
counted as the baseline counts its F lines; an item that is not a cell (`*`, `!Panic`) counts at
the case's milestone, which for a catalog case is M2. Each row with known failures is followed by a "Failing cases in <row>" list of case id and
the row's known-failure item count there (most first, then by id; also
`probe_milestones[].failing_cases`).
The console summary prints the M0–M2 row.
The rows are exit measures; `eval-check` is the gate.

## Milestone function scope
`crates/rustsheet-eval/src/function_scope.rs` scores the `ironcalc-calc` cells by the functions
their formulas call (T-1472, T-2111). report.md has a "Milestone function scope (ironcalc-calc)"
section (report.json: `function_scope.rows`) with one cumulative row for each report row of the probe
milestones (M2, M5 to M11; the same list, plus any other catalog milestone; M0 and M1 functions count in M2): a cell is in the row of `Mk` when every
function its formula calls (an array member: its anchor's; plus the functions of any defined name it
calls, up to 16 levels) has a catalog milestone of at most `k`. A function-free formula is in every
row and a function the catalog does not list counts as in scope. Each row has scored cells,
bit-exact, deviations and failing; the failing cells are listed once, under the first row that has
them ("Newly failing at Mk"). Cells with neither formula nor anchor (spill children) are left out and
counted separately. The M2 row is the M2 exit measure and the M5 row the M5 one (docs/milestones.md).
The cells a decision leaves out of an exit measure are named in `eval/milestones/exit-exceptions.tsv`
(`exit_exceptions.rs`, decision 0087; one line per cell: `case!Sheet!A1`, the first milestone whose row
leaves it out, the decision, the owning task). Each row lists them as `excepted` (report.json
`function_scope.rows[].excepted`) and report.md shows `failing excl. exceptions` beside `failing`, which
keeps its meaning. `run` and `check` refuse a malformed line or a repeated cell; a listed cell that fails
in no row is listed under "Stale exit exceptions" without failing the run. The file never touches
eval/baseline or the gate.

The Phase 1 rate (`v15_checkpoint`) has its own list, `eval/milestones/phase1-exceptions.tsv`
(`phase1_exceptions.rs`, decision 0134; one line per cell: `case!Sheet!A1` of any suite, the decision
that names it as M14-only desktop evidence, the owning M14 task). A listed visibly-off Excel cell
leaves `rate_cells` and `rate_visible` of the function whose count holds it (`rate_excepted`);
`cells`, `visible`, the tier and the strict count keep it. report.md prints the number left out per
function under the Phase 1 line, and warns about a listed cell whose suite the run scored but that is
not scored or not visibly off (`stale_exceptions`). A line in a suite the run skipped (not fetched or
not selected) is listed as not checked (`unchecked_exceptions`), never as stale. The same validation
refuses a malformed line in `run` and `check`.

The rate also leaves out the cells only open M14 tasks own (decision 0143), with no hand-kept line:
`make eval` runs `scripts/desktop_owned.py`, which applies exit_owners.py's `owned` rule to every
cell `F` line of eval/baseline and writes `case!Sheet!A1<TAB>T-<owner>` lines to the git-ignored
`eval/runs/desktop-owned.tsv`, then `rustsheet-eval run --desktop-owned` reads it
(`Phase1Exceptions::load_desktop_owned`). Such a visibly-off cell leaves `rate_cells` and
`rate_visible` as `desktop_owned` (per function and in total; report.md's "M14-owned, left out"
column); a cell both lists hold counts once, as `rate_excepted`. A generated cell that is not
visibly off is not stale. Without the flag, `run` leaves no such cell out.

## Catalog coverage per milestone
`crates/rustsheet-eval/src/catalog_coverage.rs` (T-2112, T-2175) adds a "Catalog coverage per milestone"
section to report.md (report.json: `catalog_coverage`), one row per function milestone. A function is
`covered` by the criterion of decision 0085 item 4: it has (a) the evidence floor of Excel cells and
(b) a scored Excel-backed cell in a generated suite of the milestone, or it is volatile or
`oracle: none` and has (a) alone. `both` keeps its meaning (a and b); `covered` adds the exempt
functions, which report.md lists apart as "Mk exempt from b". The row is at 100% when `covered` equals
`functions`. The table is an exit measure, not a gate; M6 inherits the rule.

## Milestone exit
`crates/rustsheet-eval/src/milestone_exit.rs` (T-2177) adds a "Milestone exit (decisions 0085, 0114)"
section to report.md (report.json: `milestone_exit`) with one row each for M5 and M6 (M7 to M11 need
their own decision first). It reads four rows the report already computes and scores nothing itself:
catalog coverage (`covered == functions`), the `gen-m<k>` suite (at least 20,000 scored cells, `cells -
deviations`, with at least 0.999 of them not visibly off; visibly-off cells print beside the allowance
`floor(0.001 * scored)`), `ironcalc-calc` visibly-off cells excluding exit exceptions (0), and the
probe row's visibly-off cells (0). Decision 0114 measures Phase 1 at visible parity; the bit-level
counts (`gen_known_failures`, `calc_failing`, `probe_known_failures`) print in brackets for Phase 1b.
A missing input prints `n/a` and is never `met`; `exit` is `met` only when all four are. It is an exit
measure, not a gate. The M2 row (decision 0135) has no coverage criterion (its counts print, its
verdict is `n/a` and `exit` ignores it), needs 10,000 scored `gen-m2` cells with none visibly off,
and reads the `M0-M2` probe row (`probe_row`). Each row lists its gen suite's visibly-off cells
(`gen_visibly_off_cells`) and each probe row its own (`visibly_off_cells`), so
`scripts/exit_owners.py` lists exactly the visibly-off exit cells without re-deriving closeness.
The M9 row (decision 0146 item 3, T-0095) requires no suite (`gen_suite` is empty and prints `none
required`); its ironcalc-calc measure is a rate: the function-scope row less its deviations and exit
exceptions (`calc_scored`) with at most `floor(0.0005 * calc_scored)` visibly off (`calc_allowance`),
and every `v15_checkpoint` function of catalog milestone M9 must be `meets_rate` (`rate_functions`,
`below_rate`, `rate_met`; `below_rate_functions` names the others, whose `attributed_off` cells
`exit_owners.py` lists as `rate:<FUNCTION>`). Rates are over a denominator of 10,000 so that 99.95% is
one `meets_rate` call. The M8 row (decision 0146 item 2, T-0096) reads gen-m8a (`gen_suite`), and as
`more_suites` gen-m8b and the `poi M8 subset` (the report's `poi_m8_subset`, at 99.5% with no
`blocked_cases`, the crashed or errored poi cases holding a subset formula, listed as `case!*`).

The "Fifteen-digit parity checkpoint" section (`v15_checkpoint`, decision 0094) also prints the
Phase 1 line of decision 0114: the in-scope functions that are covered and have at most one cell in
a thousand visibly off, with each function's within-15-digit share. Decision 0118 counts those cells
(T-0080): each function's `rate_cells` and `rate_visible` are its `attributed` cells, or its `excel`
cells below 20 attributed cells (`attribution`), and `meets_rate`, `at_rate`, `phase1_met` and the
Phase 1 line follow them. Coverage (`covered`) still counts every Excel cell that calls the function
(decision 0094). The stricter share, `cells` against the blamed `visible`, stays as information:
`strict.meets_rate`, `at_strict_rate` and a line labelled "Information, gates nothing".

## CLI
```
rustsheet-eval suites | fetch [--suite S] | run [--suite S] [--filter X] | check [--evidence-floor] | update-baseline [--allow-regressions]
               | evidence-floor [--update [--allow-increase]]
               | show <case-id> [--all] | snapshot <file.xlsx>
```
`snapshot` is the **only** xlsx value parser in the repo. Python tools call it.
