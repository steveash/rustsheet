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
| `cached_overrides` | xlsx: optional file (relative to the manifest) replacing single cached values that are not en-US evidence with cells of committed `excel-graph`/`excel-desktop` sidecars (`overrides.rs`, decision 0032) |
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
  formula Excel accepts as a name body, e.g. an intersection; sheet-scoped names are not supported).
- Optional `"sheet_ops"` lists sheet edits made after the workbook loads and before it is
  recalculated (T-1608): `{"op":"add","sheet":"New"}` appends a sheet, `{"op":"rename",
  "sheet":"Data","to":"Inputs"}` renames one (formulas that refer to it are rewritten), and
  `{"op":"move","sheet":"Data","to":0}` moves one to a 0-based position. Any other op or key is
  refused. spec2xlsx records them as docProps custom properties `rustsheet.sheetop.<n>`. The
  excel-graph backend replays them in the session after the typed entries and keeps its canary
  sheet last: an added sheet goes just before the canary, and a move may not reach the canary's
  position. The other backends refuse such cases. The engine adapter applies each op as its own
  `Workbook::edit` (`Txn::add_sheet`, `rename_sheet`, `move_sheet`). Keys in `expected` and
  `check` use the sheet names after the edits.

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
`{format, source{kind, application, version, produced_at, input_sha256}, sheets{Sheet:{A1:value}}}`.
`input_sha256` hashes the `spec2xlsx` output, which is deterministic since T-0015 (creation time pinned in `docProps/core.xml`). Sidecars filled before T-0015 carry hashes of non-reproducible bytes: treat them as legacy; they refresh on the next fill, and values do not depend on the hash.
Oracle values override hand expectations. `libreoffice` results are never written as sidecars.
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

| Verdict | Meaning |
|---|---|
| `pass` | bit-identical (−0 == +0), same text/bool/error |
| `numeric_ulp` | ≤ 4 ULP apart (operation order / rounding) |
| `numeric_close` | relative error ≤ 1e-9 (algorithm/precision difference) |
| `numeric_mismatch`, `text_mismatch`, `bool_mismatch` | wrong value |
| `error_mismatch` | different error kinds |
| `unsupported` | engine returned `#NAME?`/`#N/IMPL` where Excel did not |
| `type_mismatch` | different value types |

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
  functions leave the function tables and get their own section with their reason; the evidence
  section lists in-scope catalog functions with no Excel evidence, the `oracle: none` ones that
  need another evidence source, and, by catalog category, the ones below the evidence floor
  (fewer than 20 Excel cells; T-0056, decision 0062). Suite cell counts and baselines still cover every checked cell.
- `report.json`: the same data, machine-readable.
- `eval/baseline/<suite>.txt`: known failures (see `docs/process/quality-gates.md`).

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

## CLI
```
rustsheet-eval suites | fetch [--suite S] | run [--suite S] [--filter X] | check | update-baseline [--allow-regressions]
               | evidence-floor [--update [--allow-increase]]
               | show <case-id> [--all] | snapshot <file.xlsx>
```
`snapshot` is the **only** xlsx value parser in the repo. Python tools call it.
