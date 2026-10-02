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
- `poi`: Apache POI Excel-authored xlsx/xlsm (external, fetched with `rustsheet-eval fetch --suite poi`; 3.8k cells).
- `smoke`: hand-written core semantics.
- `probes`: Excel-behavior questions awaiting the oracle.
- `gen-m2`: generator output, pending oracle.

## Case formats
**`rustsheet-case/v1`** (`*.case.json`):
```json
{"format":"rustsheet-case/v1","description":"...","tags":["M1"],
 "sheets":[{"name":"Sheet1","cells":{"A1":1,"A2":"text","A3":"=A1+1","A4":true,"A5":{"t":"e","v":"#N/A"}}}],
 "expected":{"Sheet1!A3":{"t":"n","v":2}}, "expected_source":"hand", "check":["Sheet1!A3"]}
```
Input conventions:
- JSON number is a number and JSON bool is a boolean.
- A string starting with `=` is a formula, written with **dynamic-array (M365) semantics**.
- Any other string is text. Use the tagged form for text that starts with `=` or for errors.
- The empty string `""` is materialized as `=""`.

The same conventions are implemented by `tools/.../spec2xlsx.py` (for Excel) and by
`engine.rs::evaluate_spec` (for RustSheet).

**Values** (shared by expectations, oracle files and reports):
- `{"t":"n","v":1.5}` number
- `{"t":"s","v":"x"}` text
- `{"t":"b","v":true}` boolean
- `{"t":"e","v":"#DIV/0!"}` error
- `{"t":"empty"}` blank

JSON numbers are written shortest-round-trip, so they are exact.

**`rustsheet-oracle/v1`** sidecar (`<case>.oracle.json`): `{format, source{kind, application,
version, produced_at, input_sha256}, sheets{Sheet:{A1:value}}}`. Oracle values override hand
expectations. `libreoffice` results are never written as sidecars.

## Scoring
Every formula cell (including array-formula members) that has an expectation is checked.
Formulas calling NOW/TODAY/RAND/RANDBETWEEN/RANDARRAY are skipped (counted as `skipped_volatile`).

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
  ranked by failing cells; failing examples per function. This is the hill-climbing to-do list.
- `report.json`: the same data, machine-readable.
- `eval/baseline/<suite>.txt`: known failures (see `docs/process/quality-gates.md`).

## CLI
```
rustsheet-eval suites | fetch [--suite S] | run [--suite S] [--filter X] | check | update-baseline [--allow-regressions]
               | show <case-id> [--all] | snapshot <file.xlsx>
```
`snapshot` is the **only** xlsx value parser in the repo. Python tools call it.
