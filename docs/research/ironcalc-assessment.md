# IronCalc architecture assessment (for the RustSheet fork)

> Research snapshot produced at project inception (2026-10-01) by research agents. Treat as background, not as normative spec; normative docs live in `docs/architecture/` and `docs/process/`. Update via a PR when findings change.

- **Snapshot assessed:** `e29e7b5c5a36b40c8d3b3c49b8731ac426f43606` ("update: improve testing for TIME, HOUR, MINUTE, SECOND (#1456)", 2026-09-30). Shallow clone, 50 commits visible.
- **Repo root:** `<upstream IronCalc checkout>`. All paths below are relative to it.
- **Toolchain used:** rustc/cargo 1.97.0. Host: 4 vCPU Intel Xeon @ 2.10 GHz.
- **Conventions:** "Verified" means I ran it or read the code. "(Speculative)" or "(unverified)" marks inference. When I say Excel does something, I mean documented or well-known Excel behaviour, and I flag it wherever I am not certain.

---

## 0. Executive summary

- IronCalc is a mature, actively developed engine. The core crate `ironcalc_base` is 129k Rust LOC: about 72k of production code and about 57k of tests.
- It already has several things we want:
  - 496 function variants.
  - Dynamic arrays with spill, plus CSE arrays.
  - LET, LAMBDA, MAP, REDUCE, SCAN, BYROW, BYCOL and MAKEARRAY.
  - Structured (table) references and defined names.
  - Conditional formatting and number formatting.
  - An xlsx reader and writer.
  - A test harness that compares against cached values in Excel-saved workbooks.
  - pyo3 Python bindings.
  - `unsafe`-free code, and a clean clippy run.
- **There is no dependency graph.**
  - `Model::evaluate()` recomputes every formula on every call. It walks every cell of every sheet, constants included, using a demand-driven recursive pass.
  - Nothing persists between calls except the order of dynamic-array anchors.
  - `UserModel` calls `evaluate()` after every edit.
  - Measured: about 0.49 s per `evaluate()` for 1M trivial formulas, even when nothing changed.
- The evaluator is **pull-based**: it recurses into precedents as it reads them. Arguments arrive at functions as AST nodes, so `IF` and similar functions short-circuit. That pull model fits lazy, on-demand evaluation well. But the dirty-tracking layer, the dependency graph and volatile tracking all have to be built from scratch.
- Parity is good but not bit-exact, and the stated philosophy is sometimes "better than Excel" rather than "same as Excel":
  - The harness uses a relative tolerance of 5e-8.
  - My spot probes found several mismatches: number-to-text conversion, `ROUND(1.005,2)`, `DATE(1900,2,29)`, and missing 3D, union and intersection references.
- Recommendation:
  - Fork, and keep: the parser/AST, the function library, the formatter, the xlsx import and export, and the test corpus.
  - Rewrite first: the evaluation core (a DAG plus dirty propagation), cell value storage, and row/column structural edits.
  - Next: remove the `'a` lifetime from `Model`, and replace `String` errors with typed errors.

---
## 1. Workspace layout

The root `Cargo.toml` declares workspace members `base`, `xlsx`, `bindings/wasm`, `bindings/python` and `bindings/nodejs`. It excludes `generate_language`, `generate_locale` and `webapp/app.ironcalc.com/server`. The release profile sets `lto = true`.

| Crate (path) | Package name | Purpose | Rust LOC (`wc -l`) | Key deps | Keep for headless? |
|---|---|---|---|---|---|
| `base/` | `ironcalc_base` 0.8.3 | The engine: data model, parser, evaluator, ~496 functions, number formatter, styles, conditional formatting, undo/redo `UserModel` | 129,142 in 497 files (about 72.2k production, about 57k tests) | serde, ryu, chrono, bitcode 0.6, csv, statrs 0.18; native: rand, chrono-tz, regex; wasm: js-sys, wasm-bindgen, regex-lite | **Yes (core)** |
| `xlsx/` | `ironcalc` 0.8.3 | xlsx import (quick-xml 0.42, zip 0.6) and export; `.icalc` (bitcode) persistence; the `compare` harness; binaries `test` and `xlsx_2_icalc`; `build.rs` generates one test per xlsx file | 9,199 in `src` + 1,041 in `tests/*.rs`; 248 test `.xlsx` files (4.3 MB) | zip, quick-xml, serde_json, thiserror, itertools, chrono, bitcode | **Yes** |
| `bindings/python/` | `pyroncalc` (cdylib `_ironcalc`) | pyo3 0.27.2 / maturin bindings: raw `Model` plus `UserModel` | 2,026 | pyo3, pythonize 0.27, bitcode | **Yes (rewrite the API surface)** |
| `bindings/wasm/` | `wasm` 0.8.4 | wasm-bindgen bindings for the web app (158 `#[wasm_bindgen]` items) | 1,391 | wasm-bindgen, serde-wasm-bindgen | Drop |
| `bindings/nodejs/` | `ironcalc_nodejs` | napi-rs 3 bindings (about 215 `#[napi]` items) | 2,202 | napi, napi-derive | Drop (Java later is a separate effort) |
| `generate_language/` | (excluded) | Builds `base/src/language/language.bin` (23.8 KB, bitcode) from `src/languages.json` (en, it, fr, de, es function/error names) | 543 | serde_json, bitcode | Keep as a tool, or replace with `build.rs`/const tables. Optional |
| `generate_locale/` | (excluded) | Builds `base/src/locale/locales.bin` (2.9 KB) from CLDR JSON. Locales: en, en-GB, de, es, fr, it | 591 | clap, serde_json, bitcode | Keep only if we need non-en locales. Note the CLDR data licence (§11) |
| `webapp/` | TS/React apps + Rocket server | `webapp/IronCalc` (React widget, about 33k TS/CSS/MD lines), `app.ironcalc.com` (frontend about 5.5k + Rocket/sqlite server, 227 Rust LOC), `embed.ironcalc.com` | 227 Rust + about 38.9k TS etc. | rocket, rocket_db_pools | **Drop** |
| `docs/` | VitePress site + typedoc | User docs, one markdown page per function under `docs/docs.ironcalc.com/src/functions/...` | about 9.5k lines | — | Drop, but mine it for function semantics notes |

### Other repo-level files

- `Dockerfile` and `docker-compose.yml`: drop.
- `Makefile`: the lint and test entry points.
- `codecov.yml`.
- `.github/workflows/*`: see §9.

### What I would drop for a headless engine

- `webapp/`, `docs/`, `bindings/wasm`, `bindings/nodejs`, the Docker files.
- The `UserModel` UI parts in `base/src/user_model/ui.rs` (1,088 LOC): selection, arrow keys, scroll and window size.
- From `base/src/types.rs`: `WorkbookView` and `WorksheetView`.
- These are probably also unneeded for an engine:
  - `user_model/autofill.rs` (737 LOC)
  - `user_model/sequence_detector.rs` (805 LOC)
  - `user_model/clipboard.rs` (769 LOC)
  - `user_model/border.rs` (502 LOC)
- The wasm-only `cfg(target_arch="wasm32")` code paths in `base`, for example `regex_lite` in `expressions/parser/static_analysis.rs:5-6` and the clock code in `model.rs:52-70`.
- `base/build.rs` shells out to `git describe` to feed `INFO("release")`. Adjust it for the fork.

### Non-test `base/src` breakdown (lines)

| Area | Lines |
|---|---|
| `functions/` | 35,256 (includes some tests) |
| `expressions/` (lexer, parser, stringify, static analysis, move_formula) | 12,941 (includes about 5k of parser/lexer tests) |
| `user_model/` | 8,660 |
| `formatter/` (Excel number-format engine) | 3,720 |
| `model.rs` | 3,870 |
| `actions.rs` (insert/delete/move rows and columns) | 1,826 |
| `conditional_formatting.rs` | 1,757 |
| `evaluation.rs` | 1,114 |
| `types.rs` | 977 |
| `new_empty.rs` | 997 |
| `worksheet.rs` | 941 |
| `styles.rs` | 747 |
| `sheet_data.rs` | 400 |

---

## 2. Evaluation model (critical)

### 2.1 Facts

- **There is no persistent dependency graph and no dirty tracking.** `Model::evaluate()` (`base/src/evaluation.rs:486-514`) does a full recalculation of the whole workbook on every call.
- **Single entry point.** `Model::evaluate()` runs these steps:
  1. `sync_anchor_order()` (`evaluation.rs:625-632`).
  2. Snapshots dynamic spills.
  3. Loops `run_pass()` until a pass completes without a "restart".
  4. Re-evaluates all conditional formatting (`evaluation.rs:513`).
- **`run_pass` (`evaluation.rs:658-688`)**
  - Clears all per-pass state: the `cells` state map, `stack`, `seen`, `seen_ranges`, `links`, LET variables and lambdas.
  - Then calls `evaluate_cell` on every dynamic-array anchor (in the remembered `anchor_order`) chained with **every position that holds a cell in every sheet** (`all_positions()` → `get_all_cells()`, `model.rs:2931`).
  - That includes constants and empty styled cells. Each one is fetched and its value is materialised; strings are **cloned** (`get_cell_value`, `model.rs:1404-1420`).
- **Per-pass state** (`Evaluation`, `evaluation.rs:375-425`):
  - `cells: HashMap<(sheet,row,col), CellState{Evaluating|Evaluated}>` with a custom multiply-rotate hasher (`KeyHasher`, `evaluation.rs:57-79`).
  - `stack: Vec<CellReferenceIndex>`.
  - `circular: HashSet`.
  - `seen` / `seen_ranges`: records of positions read as empty or occupied, used only for spill-conflict detection.
  - The only state that persists across evaluations is `anchor_order: Vec<CellReferenceIndex>`.
- **Demand-driven recursion (the "pull" model)**
  - Evaluating a formula cell walks the AST (`Model::evaluate_node_in_context`, `model.rs:595`).
  - Whenever a `ReferenceKind` node is met, it calls `evaluate_cell` on the target (`model.rs:609-630`).
  - The target is evaluated recursively if it is a formula not yet evaluated in this pass (`evaluate_formula_cell`, `evaluation.rs:783-822`).
  - Ranges are returned unevaluated as `CalcResult::Range{left,right}` (`model.rs:637-676`). Each function decides how to iterate them, typically cell by cell with `evaluate_cell` (for example `fn_sum`, `functions/math_and_trigonometry/mathematical.rs:257-330`).
  - The visit order is therefore a dynamic topological order discovered fresh on every pass.
  - Dynamic references (`INDIRECT`, `OFFSET`, `INDEX` returning ranges) are handled correctly for free.
- **Laziness inside a formula.** Functions receive `&[Node]` (unevaluated arguments) (`functions/mod.rs:2308`). `IF` evaluates only the chosen branch for a scalar condition (`functions/logical/mod.rs:111-136`); `IFERROR`, `CHOOSE` etc. follow the same pattern. Across the workbook there is **no laziness**: everything is computed on `evaluate()`. Reading a value (`get_cell_value_by_index`) returns the stored, possibly stale, value and never triggers evaluation.
- **Cycle detection**
  - Reading a cell whose state is `Evaluating` closes a cycle. `mark_cycle` (`evaluation.rs:963-972`) marks every stack entry from that cell to the top as circular. All of them store **`#CIRC!`**, an IronCalc-specific error (`expressions/token.rs:84-97`).
  - Excel instead shows a circular-reference warning and keeps 0 or the last value.
  - There is **no iterative calculation**: no `iterate`/`calcPr` support (grep found none, and the exporter writes an empty `<calcPr/>`, `xlsx/src/export/workbook.rs:121`).
- **Stack-depth protection**
  - Recursion is bounded by `DEFAULT_MAX_DEPTH = 64` formulas and `DEFAULT_MAX_STACK = 256 KiB` (`evaluation.rs:432-433`, `too_deep` at `:829`).
  - Past the bound the recursion "unwinds": formulas in progress stay `Evaluating` on `stack`. The outermost frame then replays the stack top-down (`replay`, `evaluation.rs:841-866`).
  - This is how a 1M-row backwards chain (`xlsx/examples/deep_chain.rs`) avoids stack overflow.
- **Dynamic arrays and spill**
  - Anchors are `Cell::ArrayFormula{kind: Dynamic, r:(w,h)}`; spilled cells are `Cell::SpillCell{a: anchor}` (`types.rs:388-423`).
  - Because the spill footprint is known only after running, a pass records what each read saw (`record_seen`, `clip_to_used_area`).
  - When an anchor's spill would contradict an earlier read, the pass is **abandoned and restarted** with that anchor ordered first (`spill_contradicts_a_read`, `evaluation.rs:1063-1105`). The learned facts live in `RestartLog` (`:246-370`).
  - This is a clever, self-documented algorithm (module doc at `evaluation.rs:1-39`). A worst case of O(n²) restarts is possible for n anchors.
  - The "every pass starts from the same sheet" invariant is maintained by snapshotting and restoring all dynamic spills (`dynamic_spills` / `restore_dynamic_spills`, `:560-604`).
- **Volatile functions.** There is no concept of volatility in the engine. Grep for "volatile" in `base/src` returns nothing outside tests. `RAND`, `NOW` etc. simply re-run on every full pass. The xlsx importer only special-cases Excel's `ca="1"` volatile spill placeholders (`xlsx/src/import/sheet_data.rs:407-445`). A `mock_time` feature pins `NOW`/`TODAY` for tests (`base/src/mock_time.rs`, `xlsx/src/mock_time.rs`).
- **Automatic evaluation.** `UserModel` calls `evaluate_if_not_paused()` → `Model::evaluate()` after essentially every mutating action, for example `set_user_input` (`user_model/common.rs:427-490`, `:2312-2316`). `pause_evaluation` / `resume_evaluation` exist (`:366-380`).

### 2.2 Exact call path: setting a cell → computed value

```
UserModel::set_user_input(sheet,row,col,&str)            user_model/common.rs:427
 ├─ validate row/col, clone old cell for undo Diff::SetCellValue
 ├─ set_user_input_with_link_diffs                         common.rs:~495
 │   └─ Model::set_user_input(sheet,row,col,String)        model.rs:2317
 │       ├─ prepare_cell_for_user_input                    model.rs:2210  (refuse merged/CSE cells; clear dynamic spill area)
 │       ├─ empty → cell_clear_contents
 │       ├─ leading "'" → set_cell_with_string (quote-prefix style)
 │       ├─ formula (=, or +/- non-number) → set_cell_with_formula   model.rs:2600
 │       │    ├─ self.parser.parse(formula, CellReferenceRC)  (A1 mode, user locale/language; retries with ")" appended)
 │       │    ├─ run_static_analysis_on_node → StaticResult (Scalar | Array | Range | Unknown)
 │       │    ├─ shared_formula_index: to_rc_format(node) → dedupe via HashMap → push String + Arc<Node>   model.rs:2563-2598
 │       │    └─ worksheet.set_cell_with_formula(f_index) or set_cell_with_dynamic_formula (if not scalar)
 │       │    └─ compute_node_units → maybe auto-apply a number format (%, currency, date)
 │       └─ else: parse_formatted_number (locale-aware, currencies) → NumberCell (+format);
 │                bool → BooleanCell; error literal → ErrorCell; else SharedString (+ auto-link URLs)
 ├─ evaluate_if_not_paused → Model::evaluate()              evaluation.rs:486
 │    └─ loop run_pass():  for cell in anchor_order ++ all_positions():
 │         evaluate_cell(cell)                               evaluation.rs:693
 │           ├─ SpillCell → evaluate_spill_cell (may Restart)
 │           ├─ constant → get_cell_value (clone)
 │           └─ formula → evaluate_formula_cell → run_top → compute_formula
 │                 └─ evaluate_node_in_context(Arc<Node>, cell)   model.rs:595
 │                       ├─ ReferenceKind → evaluate_cell(target)  (recursion)
 │                       ├─ RangeKind → CalcResult::Range (lazy)
 │                       └─ FunctionKind → evaluate_function → fn_xxx(args:&[Node])
 │                 └─ set_cells_with_result (writes FormulaValue / spill cells)  model.rs:922
 │    └─ evaluate_conditional_formatting()  (all rules, rebuilt from scratch)
 ├─ auto row-height adjustment (Diff::SetRowHeight)
 └─ push_diff_list → History + send_queue
Read: Model::get_cell_value_by_index / get_formatted_cell_value → reads stored v (no evaluation)
```

### 2.3 Measured cost

I used a micro-benchmark crate (`scratchpad/bench`, release build, no LTO) against the raw `Model`:

| Scenario | Result |
|---|---|
| 1,000,001 formulas (500k `=A{r}*2`, 500k `=$A${r}+1`, one `SUM`) plus 1M constants: first `evaluate()` | 0.492 s |
| Same workbook, `evaluate()` after changing one input cell | 0.491 s (full recalc) |
| Same workbook, `evaluate()` with **no change** | 0.485 s |
| 200k formulas: evaluate / after one change / no change | 71 / 62 / 55 ms |
| N cells each `=SUM($A$1:$A$N)` | N=2k: 44 ms; 4k: 179 ms; 8k: 758 ms (O(N²), about 12 ns per cell read) |
| Same with `=SUM(A:A)` | 70 / 280 / 1208 ms. Whole-column ranges call `Worksheet::dimension()`, which **scans every cell of the sheet** each time (`worksheet.rs:673-698`, "FIXME: It's probably better to just track the size") |

### 2.4 Gap versus RustSheet's target (full DAG, minimal recompute, lazy)

| Needed | IronCalc today | Effort / notes |
|---|---|---|
| Persistent precedent→dependent graph, including range dependencies (a range index such as per-column interval trees, R-trees or block buckets) | None; dependencies are rediscovered every pass | New subsystem. The `record_seen` / `record_seen_empty_range` hooks (`evaluation.rs:976-1023`) already show where reads are intercepted, so they are a natural place to record dynamic edges |
| Dirty marking and propagation on edit; recalc only the dirty closure | None; every pass touches every cell | New |
| Volatile set (RAND, NOW, TODAY, OFFSET, INDIRECT, CELL, INFO, RANDARRAY, RANDBETWEEN …) always dirty | None | New; needs a per-`Function` flag |
| Dynamic dependencies (INDIRECT, OFFSET, INDEX:INDEX, structured references after table resize) | Correct "for free" via pull recursion | Must be recorded at runtime and treated as volatile or rebound each run |
| Lazy, on-demand evaluation (compute when a cell is read) | Reads never evaluate; values may be stale until `evaluate()` | The pull evaluator already supports evaluating a single cell outside a pass (`in_pass == false` path: field at `evaluation.rs:400-404`, fallback branch at `:765-778`), so lazy reads can be built on it |
| Spill-region tracking in the graph | Restart-based algorithm per pass; anchor order persisted | Keep the semantics but move them into graph maintenance (spill range = dynamic output edge) |
| Excel circular-reference semantics and iterative calculation (max iterations / max change) | `#CIRC!` error, no iteration | New, and a parity requirement |
| Parallel recalc (independent subgraphs) | Single-threaded, `&mut Model` everywhere | Requires splitting `Model` state (parser, LET variable stack, lambdas, links are mutable model fields, `model.rs:176-217`) |
| Incremental conditional formatting | Full re-evaluation after every pass | Optional |

---

## 3. Data model and memory

### 3.1 Workbook, worksheet and cell (`base/src/types.rs`)

**`Workbook`** (`types.rs:122-135`) holds:
- `shared_strings: Vec<String>`
- `defined_names: Vec<DefinedName{name, formula: String, sheet_id}>`
- `worksheets: Vec<Worksheet>`
- `styles: Styles`
- `tables: HashMap<String, Table>`
- `views`, `theme`, `settings{tz, locale}`, `metadata`

Everything derives `bitcode::{Encode, Decode}`; that is the `.icalc` format. I found no explicit schema version field.

**`Worksheet`** (`types.rs:258-280`) holds:
- `sheet_data: SheetData`
- `shared_formulas: Vec<String>`: the per-sheet formula table
- `cols: Vec<Col>` (min/max ranges)
- `rows: Vec<Row>`: row metadata such as height and style; this is not the cell storage
- `merged_cells: Vec<MergedCell>`
- `comments`
- `conditional_formatting: Vec<…>`
- `links: HashMap<(i32,i32), Link>`
- frozen panes and views

**`Cell` enum** (`types.rs:378-423`). Every variant carries a style index `s: i32`:
- `EmptyCell`
- `BooleanCell{v}`
- `NumberCell{v: f64}`
- `ErrorCell{ei}`
- `SharedString{si: i32}`
- `CellFormula{f: i32, v: FormulaValue}`
- `ArrayFormula{f, r:(w,h), kind: Cse|Dynamic, v}`
- `SpillCell{a:(row,col), v: SpillValue}`

**`FormulaValue`** (`types.rs:322-336`) is one of:
- `Unevaluated`
- `Boolean`
- `Number`
- `Text(String)`
- `Error{ei, o: String /*origin "Sheet3!C4"*/, m: String /*message*/}`

Error results therefore allocate two strings per cell, and every read of an error cell re-parses the origin string (`model.rs:1466-1476`). This is a hot-spot and memory-bloat candidate.

**Measured sizes:**

| Type | Size |
|---|---|
| `size_of::<Cell>()` | **80 B** |
| `(i32, Cell)` stored in a row | **88 B** |
| `FormulaValue` | 56 B |
| AST `Node` | 80 B |

### 3.2 Sparse storage (`base/src/sheet_data.rs`)

- The module doc is at lines 1-28. **No hashing** is involved:
  - `SheetData{chunks: Vec<Option<Box<Chunk>>>, row_count}`.
  - A chunk covers `CHUNK_ROWS = 1024` rows. It is a `Vec<Option<Row>>` always 1024 long, about 24 KB each, allocated on first use.
  - A `Row` is `Vec<(i32 column, Cell)>` sorted by column. Search is linear up to 8 entries, then binary (`:35-109`).
- Iteration comes out in natural (row, column) order without sorting.
- **Memory measured** (VmRSS deltas, release):
  - 100k numbers in a single column: about **411 B per cell**; 500k: about 396 B per cell. The cause is Vec growth: the first push to a row allocates capacity 4 × 88 B = 352 B, plus 24 KB per chunk.
  - Filling columns B–D on the same rows then costs about 0 extra, because the capacity is already there.
  - Effective cost is therefore about 88 B per cell in dense rows and about 400 B per cell for single-cell rows.
  - Excel's in-memory footprint is far smaller (unverified for current builds). For RustSheet, a columnar or block layout with a compact 16-byte value plus side tables would be a large win.

### 3.3 Shared strings

- Text is deduplicated. `Model::set_cell_with_string` (`model.rs:2673-2703`) looks the string up in `Model.shared_strings: HashMap<String, usize>` and appends to `Workbook.shared_strings: Vec<String>`. Each distinct string is therefore stored **twice** (vector plus map key).
- Verified: 500k text cells with 100 distinct values gave `shared_strings.len() == 100`.
- The table is **append-only**: I found no garbage collection of unused strings.
- Reading a string cell during evaluation clones the `String` (`get_cell_value`, `model.rs:1418-1420`; `evaluate_range`, `:1521-1553`).

### 3.4 Styles

- `Styles` holds `cell_xfs`, `fonts`, `fills`, `borders`, `num_fmts`, `cell_style_xfs` and named `cell_styles`.
- Cells hold an index into `cell_xfs`.
- Deduplication is done by **linear scan**:
  - `get_style_index` (`styles.rs:145-172`) rebuilds a full `Style` (cloning font, fill, border and alignment) for every candidate xf and compares it.
  - `get_font_index`, `get_fill_index` and `get_border_index` (`:11-34`) are also linear.
  - That is O(#styles) with allocation per lookup. It is fine for typical workbooks (2,000 restyles with 61 xfs took 5 ms) but degrades with thousands of styles. A hash-consed style table would fix it.

### 3.5 Row and column insertion/deletion (`base/src/actions.rs`)

**`insert_rows`** (`actions.rs:1005-1106`):
1. Checks that no array formula is split (`can_insert_rows`).
2. `reset_dynamic_array_spills`.
3. For every row at or below the insertion point (descending), for every cell, calls `move_cell` (`actions.rs:488-601`):
   - Clones the cell.
   - Renders its formula or value **back to localized text** (`get_cell_formula` / `get_localized_text`).
   - Re-enters it with **`set_user_input`** at the new position, which re-parses it and re-runs number and type inference.
   - Copies the style and removes the source.
4. Rewrites `rows` metadata, links and merged cells.
5. `displace_cells` (`actions.rs:399-405`): for **every cell in the workbook**, `shift_cell_formula` (`:357-393`) stringifies the formula twice (normal and displaced) and, if the two differ, calls `update_cell_with_formula`, which re-parses. The code itself says "FIXME: This is not a very performant way" (`:376`).
6. Displaces conditional-formatting ranges and formulas.

`insert_columns` (`:612-726`), `delete_rows` and `delete_columns` follow the same pattern; `move_rows_action` and `move_columns_action` are at `:1679`, `:1758`.

**Complexity:** O(cells moved × (stringify + parse)) + O(all formula cells × stringify), with large constants.

**Measured:**
- Inserting one row at the top of a 50k-row × 2-column sheet (50k formulas) took **370 ms**; deleting took 374 ms.
- A 10k-row sheet took 89 ms. That is about 3.7 µs per cell, so roughly 3–4 s per insert per 1M cells (speculative extrapolation).
- Excel adjusts references structurally, so this is a parity risk as well as a speed one. Round-tripping values through text and `set_user_input` is a source of subtle type and format changes (unverified edge cases).

The undo of row/column deletion stores row/column data; the undo of sheet deletion stores a `Box<Worksheet>` clone (`user_model/history.rs:138-153`, `common.rs:628`).

---

## 4. Formula pipeline

### Lexer (`base/src/expressions/lexer/mod.rs`, 816 LOC; plus `ranges.rs`, `structured_references.rs`, `util.rs`)

- Two modes (`mod.rs:1-30`):
  - **A1**: user/display mode, localized function names, locale-specific separators and decimals.
  - **R1C1**: internal storage mode, always English.
- Tokens are listed in `token.rs:219-263`. They include `Reference`, `Range`, `StructuredReference{table, specifier, table_reference}`, `At` (`@`), `Spill` (`#`), braces for array constants, `Backslash` and `Percent`.

### Parser (`base/src/expressions/parser/mod.rs`, 1,374 LOC)

- Hand-written recursive descent; the grammar is in the doc comment at `:1-28`. Precedence: comparison < `&` < `+-` < `*/` < `^` < unary/`%` < `:` range < `@` / `#` < primary.
- The AST is `Node` (`:144-259`, Boxed tree). Variants:
  - `ReferenceKind` / `RangeKind`, each with `absolute_*` flags
  - `WrongReferenceKind` (#REF)
  - `OpRangeKind` (dynamic `A1:INDEX(...)`)
  - `FunctionKind{kind: Function, args}`
  - `LambdaDefKind`, `LambdaCallKind`
  - `NamedFunctionKind` (unknown or user-defined name call)
  - `ArrayKind` (array constants)
  - `DefinedNameKind`, `TableNameKind`, `NamedVariableKind` (LET/LAMBDA params)
  - `ImplicitIntersection{automatic}`, `SpillRangeOperator`
  - `CompareKind`, `UnaryKind`, `ErrorKind`
  - `ParseErrorKind{formula, message, position, expecting}`
  - `EmptyArgKind`
- **Relative references are stored as offsets** from the host cell: for non-absolute rows/columns the parser subtracts `context.row` / `context.column` (`parser/mod.rs:785-848`). Evaluation adds them back (`model.rs:620-626`, `:652-676`). This is why filled-down formulas share one AST.
- **Bug found (verified):**
  - `Parser::parse` (`parser/mod.rs:340-347`) returns `parse_expr()` without checking that the lexer reached EOF, so trailing tokens are silently dropped.
  - Example: entering `=A1:B2 B1:B3` (the Excel intersection operator) stores the formula as `=A1:B2`.
- Also present:
  - `parse_at_cursor` / `formula_completion` (autocomplete).
  - `stringify.rs` (1,385 LOC): `to_rc_format`, `to_localized_string`, `to_english_string`, `to_string_displaced` for row/column shifts, and Excel export forms with `_xlfn.` prefixes (`functions/mod.rs:2124`).
  - `move_formula.rs` (528 LOC) for cut and paste.
  - `static_analysis.rs` (2,163 LOC): per-function argument signatures and `StaticResult` (`:287-298`), which decide whether a formula is a dynamic-array anchor and where implicit intersection is applied.

### Reference forms (verified by reading code and by probing with `set_user_input` + `evaluate`)

| Form | Status |
|---|---|
| A1, `$A$1`, mixed, ranges, whole column `A:A`, whole row `1:1` | Supported |
| Cross-sheet `Sheet2!A1`, quoted `'My Sheet'!A1` | Supported |
| Defined names, workbook or sheet scope, including names that hold formulas | Supported (`parse_defined_names`, `new_empty.rs:126`; `Model::new_defined_name` etc. `model.rs:3389-3600`) |
| Structured references (`Table1[Col]`, `[#Headers]`, `[@Col]`…) | Supported for tables present in `workbook.tables` |
| Dynamic range `A1:INDEX(...)`, `OFFSET`/`INDIRECT` results | Supported (`OpRangeKind`, `returns_reference`, `functions/mod.rs:2302`) |
| Spill reference `A1#`, `@` implicit intersection | Supported |
| Array constants `{1,2;3,4}` | Supported |
| **3D references** `Sheet1:Sheet3!A1` | **Not supported**: `=SUM(Sheet1:Sheet2!A1)` → `#VALUE!` |
| **Union operator** `(A1,B1)` | **Not supported**: `#ERROR!` |
| **Intersection operator** (space) | **Not supported, and silently truncated** (see the bug above) |
| **External workbook references** `[1]Sheet1!A1` | **Not supported**: `#ERROR!` (no `externalLink` handling in the xlsx importer) |
| **R1C1 entry by the user** | Not supported (`=R1C1` → `#ERROR!`); R1C1 is used only internally |

### Formula storage

- Each worksheet has `shared_formulas: Vec<String>` holding **R1C1 text**.
- `Model.parsed_formulas: Vec<Vec<(Arc<Node>, StaticResult)>>` is a parallel vector per sheet (`model.rs:182`).
- `SharedFormulaLookup{index: HashMap<String, i32>}` (`model.rs:220-245`) deduplicates.
- Cells store only `f: i32`. Identical R1C1 formulas share one parsed `Arc<Node>`, so a formula is parsed once per distinct R1C1 text.
- Measured:
  - 500k filled-down `=A{r}*2` produced `shared_formulas.len() == 1`, with RSS growth of about 0.
  - 500k distinct `=$A${r}+1` produced 500,001 entries at about **490 B per formula** (R1C1 string, plus its copy as a HashMap key, plus the Arc'd AST).
- On load, `parse_formulas` (`new_empty.rs:93-124`) parses every `shared_formulas` entry in R1C1/English mode.
- The list is **append-only**. Replaced or deleted formulas are not compacted; I found no compaction routine. The list is rebuilt only on sheet or defined-name rename (`model.rs:3579`, `new_empty.rs:499`).
- **xlsx shared formulas.** `<f t="shared" si=…>` anchors and daughters are expanded by `xlsx/src/import/sheet_data.rs:387-560`. Because storage is R1C1, Excel's shared formulas collapse naturally into one entry. The comment at `:404` warns the algorithm "could end up with repeated shared formulas". Array (`t="array"`) and dynamic-array (metadata `ca`/`cm`) formulas are imported.

---

## 5. Function coverage

**496 `Function` enum variants** (`base/src/functions/mod.rs:29-562`). The count is verified by parsing the enum and is consistent with `Function::into_iter() -> IntoIter<Function, 496>` at `functions/mod.rs:1617`. Dispatch is at `functions/mod.rs:2308`, localized names in `language.bin`, and xlsx names (with `_xlfn.`) at `functions/mod.rs:2124`.

The list below uses the grouping of the source comments. Those groups are imprecise: for example `CORREL`/`SLOPE` sit under "Database" and `SUBTOTAL` under "Engineering: Misc".


### Notable Excel functions not implemented

I checked against the English name table in `generate_language/src/languages.json`. These are **absent**:

- `AGGREGATE`
- `GETPIVOTDATA`
- `GROUPBY`, `PIVOTBY`
- `IMAGE`
- `FILTERXML`, `WEBSERVICE`
- `CUBE*`
- `RTD`
- `DBCS`, `JIS`, `PHONETIC`, `BAHTTEXT`
- `STOCKHISTORY`, `FIELDVALUE`
- `TRANSLATE`, `DETECTLANGUAGE`
- `EUROCONVERT`, `CALL`, `REGISTER.ID`
- `PY`, `COPILOT` (not checked by name; assumed absent)

Unknown functions parse as `NamedFunctionKind` and evaluate to `#NAME?`. Verified: `=AGGREGATE(9,6,A1:A2)` → `#NAME?`.

Date/time (25) and Financial (55) look complete against the Excel lists I know (unverified against an authoritative list).

### Dynamic arrays, spill, LET and LAMBDA

All present:
- Spill with `#SPILL!`, CSE arrays, the `#` operator, `@`.
- `LET`, `LAMBDA` (including immediate invocation `LAMBDA(x,x+1)(2)` → 3), `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY`, `ISOMITTED`.
- `SEQUENCE`, `RANDARRAY`, `FILTER`, `SORT`, `SORTBY`, `UNIQUE`, `XLOOKUP`, `XMATCH`, `TAKE`, `DROP`, `CHOOSEROWS`, `CHOOSECOLS`, `EXPAND`, `HSTACK`, `VSTACK`, `TOCOL`, `TOROW`, `WRAPROWS`, `WRAPCOLS`, `TRIMRANGE`, `TEXTSPLIT`, `REGEX*`.

LET bindings and lambdas are held in mutable model fields (`variable_stack`, `lambdas`, `model.rs:194-203`) and are cleared every pass.

Named LAMBDAs via defined names go through `NamedFunctionKind` (I did not deep-verify this).

### Errors

`Error` (`expressions/token.rs:84-97`) has these variants:
- `REF`, `NAME`, `VALUE`, `DIV`, `NA`, `NUM`, `NULL`, `SPILL`, `CALC`
- Plus the non-Excel errors `ERROR` (`#ERROR!`), `NIMPL` (`#N/IMPL!`) and `CIRC` (`#CIRC!`)

Missing Excel errors: `#GETTING_DATA`, `#FIELD!`, `#BLOCKED!`, `#CONNECT!`, `#BUSY!`, `#UNKNOWN!` (minor).

### Excel-parity probes

I ran these through raw `Model` with `en` locale and UTC (`scratchpad/bench/src/bin/probe.rs`).

| Formula | IronCalc | Excel (expected) | Match? |
|---|---|---|---|
| `="1,000"+1` / `="$5"+1` / `="10%"+0` / `="1/2/2020"+0` | 1001 / 6 / 0.1 / 43832 | same | yes |
| `={"1,000"}+1` | `#VALUE!` | 1001 (believed; unverified) | likely no. The array arithmetic path uses Rust `str::parse::<f64>` (`base/src/arithmetic.rs:27-36`) instead of Excel's value coercion |
| `=ROUND(1.005,2)` | 1 | 1.01 (Excel rounds on the decimal representation) | **no** |
| `="a"&0.1+0.2`, `=CONCAT(0.1+0.2)` | `a0.30000000000000004`, `0.30000000000000004` | `a0.3`, `0.3` (number→text uses 15 significant digits) | **no**. `cast.rs:341` has "FIXME: I think when casting a number we should convert it to_precision(x, 15)" |
| `=DATE(1900,2,29)` | 61 (shown 3/1/1900) | 60 (Excel's fictitious 1900-02-29) | **no** |
| `=0.1+0.2=0.3`, `=10/3*3=10`, `=0.1*3=0.3` | TRUE | TRUE | yes |
| `=0.1+0.2-0.3`, `=1-0.9-0.1` | 5.55112E-17, -2.77556E-17 | same | yes |
| `=1E308*10`, `=2^1024` | `#NUM!` | `#NUM!` | yes |
| `=MOD(-1,3)`, `=ROUND(-2.5,0)` | 2, -3 | same | yes |
| `="abc"="ABC"` | TRUE | TRUE | yes |
| `=SUM(Sheet1:Sheet2!A1)`, `=SUM((A1,B1))`, `=[1]Sheet1!A1`, `=R1C1` | `#VALUE!`, `#ERROR!`, `#ERROR!`, `#ERROR!` | valid in Excel | **no** |

Other parity-relevant facts:
- `CalcResult` ordering compares numbers with an absolute `f64::EPSILON` (`calc_result.rs:51-60`).
- `to_precision(…,15)` is applied in places such as text comparison (`functions/text/common.rs:1119`) but not uniformly.
- There is no 1904 date-system support: no `date1904` handling in the importer.
- The transcendental-functions README (`base/src/functions/engineering/transcendental/README.md`) says that some platforms, Excel included, are "remarkably poor … We strive to do better". This is a philosophy clash with bit-for-bit parity, and those functions must be re-targeted to match Excel's outputs.

---

## 6. Public API surface

### `Model<'a>` (`base/src/model.rs:176`; also `actions.rs`, `styles.rs`, `new_empty.rs`, `evaluation.rs`)

- The raw API: no auto-evaluation, no history.
- **The `'a` lifetime is a wart.** The constructors take `&'a str` for the name, locale, tz and language (`new_empty.rs:615-620`), but the locale and language actually resolve to `&'static` (`locale/mod.rs:105`, `language/mod.rs:550`). The Python binding works around this by leaking every string passed in (`bindings/python/src/lib.rs:93-95`, `leak_str`).

Key methods (each returns `Result<_, String>` unless noted):

- **Construction and persistence**
  - `Model::new_empty(name: &'a str, locale_id: &'a str, timezone: &'a str, language_id: &'a str)`
  - `Model::from_bytes(&[u8], language_id)`
  - `Model::from_workbook(Workbook, language_id)` (`:1616`)
  - `to_bytes() -> Vec<u8>` (bitcode)
  - xlsx: `ironcalc::import::{load_from_xlsx, load_from_xlsx_bytes, load_from_icalc}`, `ironcalc::export::{save_to_xlsx, save_xlsx_to_writer, save_to_icalc}`
- **Input**
  - `set_user_input(sheet:u32, row:i32, col:i32, value:String)` (`:2317`; locale-aware type inference)
  - `set_user_array_formula(sheet,row,col,width,height,&str)` (`:2422`)
  - `update_cell_with_text`, `update_cell_with_number`, `update_cell_with_bool`, `update_cell_with_formula`
  - `range_clear_contents(&Area)`, `range_clear_all(&Area)`
- **Calculation**
  - `evaluate()`
  - `set_max_evaluation_depth(usize)`, `set_max_evaluation_stack(usize)`
  - `cycle_reference(...)` (`:3757`)
- **Output**
  - `get_cell_value_by_index(sheet,row,col) -> Result<CellValue,String>`, `get_cell_value_by_ref("Sheet1!A1")`
  - `get_formatted_cell_value`, `get_cell_formula`, `get_localized_cell_content`, `get_cell_type`
  - `get_all_cells() -> Vec<CellIndex>`, `is_empty_cell`
  - `get_sheet_markup` (dump)
- **Structure**
  - `insert_rows`, `insert_columns`, `delete_rows`, `delete_columns`
  - `move_rows_action`, `move_columns_action`
  - `add_sheet`, `new_sheet`, `delete_sheet`, `rename_sheet`, `move_sheet`
  - `extend_to`, `move_cell_value_to_area`
- **Names**
  - `new_defined_name(name, scope:Option<u32>, formula)`, `update_defined_name`, `delete_defined_name`, `get_defined_name_list`, `is_valid_defined_name`
- **Styles and layout**
  - `get_style_for_cell`, `set_cell_style`, row/column styles, widths and heights, hidden state, frozen panes, theme, grid lines, sheet colour and state
- **Settings and helpers**
  - `set_locale`, `set_timezone`, `set_language`, `get_fmt_settings`
  - `formula_completion(sheet,row,col,formula,cursor)`
  - `parse_reference(&str)`

### `UserModel<'a>` (`base/src/user_model/common.rs:212`; 75 `pub fn` in `common.rs`, plus `ui.rs`, `clipboard.rs`, `autofill.rs`, `border.rs`, `merged_cells.rs`, `links.rs`, `named_cell_styles.rs`, `conditional_formatting.rs`)

- Wraps `Model` with `History` (undo/redo), `send_queue` (diffs) and auto-evaluation.
- **Undo/redo:** `undo()`, `redo()`, `can_undo()`, `can_redo()` (`common.rs:325-360`). History is a list of `Diff` lists. `Diff` is a `pub(crate)` enum with **47 variants** (`user_model/history.rs:25-~350`), for example `SetCellValue{old_value: Box<Option<Cell>>}`, `DeleteSheet{old_data: Box<Worksheet>}`, row/column data.
- **Collaboration diffs:**
  - `flush_send_queue() -> Vec<u8>` (bitcode-encoded `Vec<QueueDiffs>`)
  - `apply_external_diffs(&[u8])` (`common.rs:395-425`)
  - These are operation-replay diffs, not CRDTs.
- **Evaluation control:** `pause_evaluation()`, `resume_evaluation()`, `evaluate()`.
- **Everything else:** mirrors of the Model operations, plus a UI/view API (`set_selected_cell`, `on_arrow_*`, `on_page_down`, scroll, window size), clipboard (`copy_to_clipboard`, `paste_from_clipboard`, CSV paste), autofill, borders, merged cells, named cell styles, links and CF rules.
- For a headless engine, the UI, view and selection parts are unnecessary. The history and diff machinery is useful as a starting point for transactions and undo, but it is tied to UI semantics, for example the auto row-height diff in `set_user_input` (`common.rs:465-479`).

---

## 7. Testing infrastructure

### Rust unit tests in `base`

- 145 entries under `base/src/test/` (files and subdirs such as `array_formulas`, `dynamic_evaluation`, `statistical`, `lookup_and_reference` …).
- Plus tests in `expressions/{lexer,parser}/tests`, `formatter/test` and `functions/engineering/transcendental/test_bessel.rs`.
- **2,592 `#[test]` attributes** (grep).
- The tests are hand-written: build a model with `_set("A1", "...")`, call `evaluate()`, assert against expected strings or numbers (`base/src/test/util.rs`). Expected values are typed in by developers, often taken from Excel (provenance is not recorded per test).

### xlsx tests in `xlsx/`

- `xlsx/build.rs` generates one `#[test]` per `.xlsx` file (`build.rs:33-66`) in:
  - `tests/calc_tests` (recursive; 207 files across `DATABASE`, `DATE_AND_TIME`, `ENGINEERING`, `FINANCIAL`, `INFORMATION`, `LOGICAL`, `LOOKUP_AND_REFERENCE`, `MATH_AND_TRIGONOMETRY`, `STATISTICAL`, `TEXT` and the root)
  - `tests/calc_test_no_export` (1)
  - `tests/templates` (8)
  - `tests/docs` (14, with 5 volatile or unstable files skipped)
- That gives **225 generated tests**. There are 248 `.xlsx` files under `xlsx/tests` in total.
- The harness is `xlsx/src/compare.rs`:
  - `test_file` (`:321-334`) loads the workbook twice. `model1` keeps **Excel's cached `<v>` values** from the file, so the expected values come from Excel itself. `model2` calls `evaluate()`.
  - `compare_models` then diffs every cell.
  - `test_load_and_saving` (`:336-360`) also round-trips through `save_to_xlsx` and re-evaluates.
  - Numbers are compared with `numbers_are_close`, a **relative tolerance `EPS = 5e-8`** (`compare.rs:32-46`). A workbook can override this with `METADATA!A1`.
  - The `METADATA` sheet also supplies the locale and a mocked `NOW` timestamp (`xlsx/src/util.rs`, `xlsx/src/mock_time.rs`).
  - **This is not bit-exact.** RustSheet needs an exact (ULP-0) mode plus string, format and error-type equality.
- Binary `xlsx/src/bin/test.rs` runs the comparison on any user-supplied xlsx (the "QA" tool).

### Python tests

`bindings/python/tests/`: 14 files, 111 `def test_*`. I did not run them; maturin is not installed here. `cargo check -p pyroncalc` succeeds.

### Run results (verified)

`CARGO_TARGET_DIR=scratchpad/target`, debug profile:

| Command | Result |
|---|---|
| `cargo test -p ironcalc_base --no-run` (cold build incl. deps) | 45.5 s wall |
| `cargo test -p ironcalc_base` | **2,583 passed, 0 failed, 8 ignored** (lib, 4.28 s) + 23 doc-tests passed; 13.4 s wall after build |
| `cargo test -p ironcalc` | **300 passed, 0 failed**: lib 45; `generated_tests` 225 (14.16 s); `test.rs` 18; CF 3; font/gridlines/xf/randarray/shared-volatile 1 each; doc-tests 3. 41.1 s wall including build |
| `cargo clippy -p ironcalc_base -p ironcalc --all-targets -- -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic -D warnings` | Clean (32 s) |

The 8 ignored tests are mostly number-format cases such as fractions, scientific notation and `#,##` grouping in `formatter/test/*`, plus `test_general::test_xlfn`.

---

## 8. Python bindings (`bindings/python`)

- **Technology:**
  - pyo3 0.27.2 (`extension-module`), pythonize 0.27, maturin ≥1.0.
  - `pyproject.toml`: package `ironcalc`, module `ironcalc._ironcalc`, `python-source = "python"`, `requires-python >= 3.10`.
  - Type stubs are in `python/ironcalc/_ironcalc.pyi` (212 def/class lines).
  - Sphinx docs are in `docs/`.
- **Build:** `cd bindings/python && maturin develop` (see `run_tests.sh`: venv, `pip install patchelf maturin pytest`, `maturin develop`, `pytest tests/`). CI wheels come from `PyO3/maturin-action` (`.github/workflows/pypi.yml`): Linux x86_64, x86, aarch64, armv7, s390x and ppc64le (manylinux), plus Windows x64/x86, macOS and an sdist (`pypi.yml:19-138`).
- **API:**
  - Module functions (`src/lib.rs:128-263`): `load_from_xlsx`, `load_from_icalc`, `load_from_bytes`, `create`, `create_user_model*`, `column_name_from_number`, `column_number_from_name`, `quote_name`, `get_all_timezones`, `get_supported_locales`, `test_panic`.
  - Classes:
    - `Model` (raw, `src/raw_model.rs`, about 75 methods mirroring the Rust `Model`)
    - `UserModel` (`src/user_model.rs`, about 110 methods)
    - `CellType`
    - `WorkbookError`
- Values are returned one cell at a time: `get_cell_value` returns a Python float, str, bool or error. There is **no bulk or array API**: no numpy, no Arrow, no ranges. That is a major performance gap for a Python-first product.
- The GIL is not released during `evaluate()`; there are no `allow_threads` / `detach` calls.
- `leak_str` leaks the locale, tz, language and name strings on every model creation (`lib.rs:93-95`).

---

## 9. Code quality

### Lints

- There is no `clippy.toml`, no `rust-toolchain` file, and no `[lints]` table.
- Lint policy lives in the `Makefile` `lint` target: `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic -D warnings`.
- `base/src/lib.rs:27` adds `#![warn(clippy::print_stdout)]`.
- Clippy passes clean on 1.97.

### Pattern counts in non-test code

Grep counts; files under `test/` / `tests/` and `test*.rs` excluded, so some in-file `#[cfg(test)]` modules may still be counted.

| | `base/src` (173 files, 72.2k LOC) | `xlsx/src` (32 files, 7.7k LOC) | `bindings` (10 files, 5.6k LOC) |
|---|---|---|---|
| `unsafe` | 0 | 0 | 0 |
| `.unwrap()` | 132 | 60 | 3 |
| `.expect(` | 37 | 1 | 0 |
| `panic!` | 1 | 3 | 2 |
| `unreachable!` | 3 | 0 | 0 |
| `#[allow(clippy::unwrap_used / expect_used)]` | 8 / 19 | 11 / 0 | 3 / 0 |
| FIXME / TODO | 28 / 32 | 7 / 11 | 2 / 0 |

### Other observations

- **Panics:** direct indexing such as `self.workbook.worksheets[cell_reference.sheet as usize]` in `fetch_cell` (`model.rs:1555-1560`) and `parsed_formulas[sheet][f]` will panic on bad indices.
- **Error handling:** errors are `Result<_, String>` throughout `base`, so there is no typed error enum. `xlsx` uses `thiserror` (`XlsxError`).
- **Documentation:** good-quality, extensive comments explaining algorithms, especially `evaluation.rs` and `sheet_data.rs`.

### CI (`.github/workflows`)

- `build-test.yaml`: on push or PR to main runs `make lint` (fmt, clippy, JS checks), `make test-rust` on ubuntu and macOS, `make test-js` (wasm and webapp) and `make test-python`.
- `test-coverage.yaml`: `cargo llvm-cov` excluding the bindings, uploaded to Codecov.
- `pypi.yml`, `npm.yml`, `publish-crates.yaml`, `docker.yml`: release pipelines on `v*` tags.
- There is no benchmark CI, no fuzzing and no Miri.

---

## 10. Performance notes

- **Benchmarks:** there are no criterion benches and no `#[bench]`. The only performance artifacts are `xlsx/examples/forward_chain.rs` and `deep_chain.rs` (1M-row chains timed with `Instant`). Code comments cite timings, for example `mathematical.rs:298`: "cargo test … .8 seconds with this speedup and ~3.5 seconds without". My measurements are in §2.3 and §3.
- **Hot spots** (verified by reading the code, with timings where measured):
  1. **Full recalc of everything on every `evaluate()`**, constants included, followed by full conditional-formatting re-evaluation (`evaluation.rs:486-514`, `:658-688`). About 0.49 µs per formula; no-op evaluate costs the same as a real one.
  2. **`Worksheet::dimension()` is O(cells)** (`worksheet.rs:673-698`). It is called by `clip_to_used_area` for every whole-row or whole-column range read (`evaluation.rs:1025-1061`) and by insert/delete checks. That adds about 60% overhead in the `SUM(A:A)` benchmark and makes the cost quadratic-ish.
  3. **String cloning** when reading string cells (`model.rs:1418`). Every evaluation of a string literal does `value.replace("\"\"", "\"")` (`model.rs:607`). Error values carry `String` messages and origins and re-parse the origin on read (`model.rs:1466-1476`). `fn_sum` clones `sheet_name` per reference argument (`mathematical.rs:265-280`).
  4. **Range functions iterate densely cell by cell** via `evaluate_cell` (one point lookup per position), and many functions materialise `Vec<Vec<ArrayNode>>` (`evaluate_range`, `model.rs:1521`). There is no columnar or SIMD path and no lookup caching for VLOOKUP, MATCH or XLOOKUP; the only cache I found in `functions/` is the LET binding cache. N formulas each summing N cells is O(N²) with no shared aggregation.
  5. **Row/column insert and delete** via text round-trip and re-parse (§3.5): about 3.7 µs per cell.
  6. **Style deduplication** by linear scan with clones (§3.4).
  7. **Memory:** 80-byte `Cell` enum and 88 B per stored cell. About 400 B per cell when rows are sparse (Vec capacity 4). Formula text is stored twice (Vec + HashMap key), as are shared strings.
  8. **Single-threaded:** `&mut Model` everywhere, with mutable parser, LET-stack and lambda state inside `Model`. No parallelism.
  9. **xlsx import is streaming** (quick-xml events, `xlsx/src/import/sheet_data.rs:150-260`). That is good; keep it.
- Numbers I measured:
  - Building 500k numeric cells via `set_user_input`: 0.32 s.
  - 500k identical-R1C1 formulas: 1.30 s. Parsing dominates: every `set_user_input` re-parses the A1 text and re-stringifies it to R1C1 for the lookup.

---

## 11. Recommendation

### Keep (fork as-is, then refactor in place)

- **The function library** (`base/src/functions/**`, about 35k LOC, 496 functions). This is the highest-value asset: years of parity work, including statistical distributions, financial bond math, Bessel/complex, database functions, dynamic-array functions, LAMBDA helpers and regex. Re-target any "better than Excel" implementations to match Excel.
- **Lexer, parser, stringifier and static analysis** (`expressions/`). Fix: the EOF check, plus 3D, union and intersection references, external references and R1C1 entry. Consider interning sheet names, and storing references as `(sheet_id, row, col)` with no `String` in `Node`.
- **The number formatter** (`formatter/`, 3.7k LOC) and the locale/language tables (consider dropping to `en` only at first).
- **xlsx import and export** (`xlsx/src`), and **the Excel-cached-value harness plus the 248-file corpus.** Extend the harness to an exact-equality mode, and grow the corpus aggressively; it is the main parity guardrail.
- **The `sheet_data.rs` chunked layout**, as a starting point. It is already encapsulated behind a small API (`cell`, `set_cell`, `remove_cell`, `cells`, `rows`), so it is easy to swap.
- **The spill/restart semantics and test suite** (`base/src/test/dynamic_evaluation`, `array_formulas`). Keep them as a behavioural oracle even if the algorithm moves into a DAG.

### Rewrite first (priority order)

1. **Evaluation core:**
   - A persistent dependency graph: cell→cell edges, plus a range-dependency index.
   - Dirty propagation and a topological or recursive dirty-only recalc.
   - A volatile set and dynamic-dependency recording for INDIRECT, OFFSET and spills.
   - Lazy evaluate-on-read.
   - Excel circular-reference and iterative-calc semantics.
   - Keep the pull-based `evaluate_cell` as the per-cell executor. The new layer decides which cells to run.
2. **Value and cell representation:**
   - A compact cell value (for example 16 B: tag + f64 / interned string id / error code).
   - Formula results stored separately from inputs.
   - Typed errors without per-cell `String`s.
   - Interned strings with refcount or GC.
   - Track used ranges incrementally to replace `dimension()`.
3. **Structural edits:** reference adjustment on the AST or R1C1 form instead of the text round-trip through `set_user_input`. Bulk-move cells in storage, which with chunked rows means shifting row indices.
4. **API cleanup:** remove the `'a` lifetime from `Model`; typed `Error` enums instead of `String`; a bulk range get/set API; split `UserModel` UI concerns out of the engine.
5. **Python bindings:** a new API designed for bulk I/O (buffer protocol, numpy or Arrow), releasing the GIL during recalc and file I/O.
6. **Parity fixes found in probes:** number→text at 15 significant digits, `ROUND`, 1900 leap-year serials, array-path value coercion, 1904 date system, missing functions (AGGREGATE first).

### Risks of forking

- **Upstream divergence.** IronCalc is moving fast: evaluation.rs and sheet_data.rs were clearly rewritten recently. Once we replace the evaluator, cell storage and `Model` API, upstream function fixes will no longer merge cleanly.
  - Mitigation: keep `functions/` as close to upstream as possible, behind a thin `FnContext` adapter.
  - Mitigation: track upstream `functions/**` and `xlsx/tests/**` periodically and cherry-pick.
- **Parity philosophy mismatch.** Upstream sometimes favours accuracy over Excel's actual output (Bessel README), and tests use a 5e-8 tolerance. Upstream test values cannot be treated as bit-exact truth.
- **Hidden coupling.** `Model` mixes parser state, LET/lambda stacks, links, the CF cache and the evaluator. Refactoring touches many `fn_*` signatures (`&mut self` on `Model`).
- **The data format** (bitcode-derived `.icalc`) is tied to struct layout. Treat it as internal and don't promise compatibility.

### Licensing and attribution obligations

The code is **dual-licensed "MIT OR Apache-2.0"**: `base/Cargo.toml` `license`, plus `LICENSE-MIT` and `LICENSE-Apache-2.0` at the root. The MIT copyright line is "Copyright (c) 2023 EqualTo GmbH, 2023 Nicolás Hatcher".

**There is no `NOTICE` file in the repo** (verified with `ls NOTICE*`).

We may choose either licence. If we distribute under **Apache-2.0 §4**, we must:
- (a) give recipients a copy of the Apache-2.0 licence;
- (b) make modified files carry prominent notices stating that we changed them;
- (c) retain all copyright, patent, trademark and attribution notices from the source form in our source distributions;
- (d) if the work contains a NOTICE file, include its attribution notices in a NOTICE with our distribution. N/A today, but re-check on every upstream sync.

Under Apache-2.0 we may add our own copyright statements and licence terms for our modifications. §6 grants **no trademark rights**: do not use "IronCalc" branding beyond factual attribution.

If we choose **MIT**, we must include the copyright notice and permission notice in all copies or substantial portions, including binary wheels. Practical approach:
- Keep both licence files with the original copyright.
- Add a `NOTICE`/`THIRD_PARTY` file listing IronCalc (EqualTo GmbH, Nicolás Hatcher) as the origin.
- Mark modified files.

**Third-party notices inside the tree:**
- `base/src/functions/engineering/transcendental/bessel_j0_y0.rs`, `bessel_j1_y1.rs` and `bessel_jn_yn.rs` contain Sun Microsystems fdlibm code. The notice reads "Permission to use, copy, modify, and distribute this software is freely granted, provided that this notice is preserved". It **must be preserved**.
- `generate_locale` derives locale data from Unicode CLDR. The CLDR data falls under the Unicode License, which requires its copyright and permission notice to accompany copies of the data (I believe this also covers derived `locales.bin`; unverified, check with counsel).
- Rust dependencies (chrono-tz with IANA tz data, statrs, ryu (Apache-2.0 OR BSL-1.0), regex, etc.) need a `cargo-about` / `cargo-deny` licence bundle for wheels.

---

## Appendix A: function list by source category (496)

**Logical (19)**: AND, FALSE, IF, IFERROR, IFNA, IFS, LAMBDA, LET, NOT, OR, SWITCH, TRUE, XOR, BYCOL, BYROW, MAP, REDUCE, SCAN, MAKEARRAY

**Mathematical and trigonometry (82)**: ABS, ACOS, ACOSH, ASIN, ASINH, ATAN, ATAN2, ATANH, CHOOSE, COLUMN, COLUMNS, COS, COSH, LOG, LOG10, LN, MAX, MIN, PI, POWER, PRODUCT, RAND, RANDBETWEEN, ROUND, ROUNDDOWN, ROUNDUP, SIN, SINH, SQRT, SQRTPI, SUM, SUMIF, SUMIFS, SUMX2MY2, SUMX2PY2, SUMXMY2, TAN, TANH, ACOT, ACOTH, COT, COTH, CSC, CSCH, SEC, SECH, EXP, FACT, FACTDOUBLE, SIGN, RADIANS, DEGREES, INT, EVEN, ODD, CEILING, CEILING.MATH, CEILING.PRECISE, FLOOR, FLOOR.MATH, FLOOR.PRECISE, ISO.CEILING, MOD, QUOTIENT, MROUND, TRUNC, GCD, LCM, BASE, DECIMAL, ROMAN, ARABIC, COMBIN, COMBINA, SUMSQ, MDETERM, MINVERSE, MUNIT, MULTINOMIAL, SERIESSUM, SUMPRODUCT, PERCENTOF

**Information (22)**: ERROR.TYPE, FORMULATEXT, ISBLANK, ISERR, ISERROR, ISEVEN, ISFORMULA, ISLOGICAL, ISNA, ISNONTEXT, ISNUMBER, ISODD, ISOMITTED, ISREF, ISTEXT, NA, SHEET, TYPE, SHEETS, N, CELL, INFO

**Lookup and reference (32)**: ADDRESS, AREAS, CHOOSECOLS, CHOOSEROWS, EXPAND, FILTER, HLOOKUP, HSTACK, HYPERLINK, INDEX, INDIRECT, LOOKUP, MATCH, OFFSET, ROW, ROWS, SORT, SORTBY, UNIQUE, VLOOKUP, VSTACK, WRAPCOLS, WRAPROWS, XLOOKUP, XMATCH, TRIMRANGE, TAKE, DROP, TOCOL, TOROW, TRANSPOSE, MMULT

**Dynamic array (2)**: SEQUENCE, RANDARRAY

**Text (45)**: CONCAT, TEXTSPLIT, CONCATENATE, EXACT, FIND, LEFT, LEN, LOWER, MID, REPT, RIGHT, SEARCH, SUBSTITUTE, REGEXEXTRACT, REGEXREPLACE, REGEXTEST, T, TEXT, TEXTAFTER, TEXTBEFORE, TEXTJOIN, TRIM, UNICODE, UNICHAR, CHAR, CLEAN, CODE, ASC, ARRAYTOTEXT, DOLLAR, ENCODEURL, FINDB, FIXED, LEFTB, LENB, MIDB, NUMBERVALUE, PROPER, REPLACE, REPLACEB, RIGHTB, SEARCHB, UPPER, VALUE, VALUETOTEXT

**Statistical (105)**: AVERAGE, AVERAGEA, AVERAGEIF, AVERAGEIFS, COUNT, COUNTA, COUNTBLANK, COUNTIF, COUNTIFS, MAXIFS, MINIFS, GEOMEAN, FREQUENCY, GROWTH, AVEDEV, BETA.DIST, BETA.INV, BINOM.DIST, BINOM.DIST.RANGE, BINOM.INV, CHISQ.DIST, CHISQ.DIST.RT, CHISQ.INV, CHISQ.INV.RT, CHISQ.TEST, CONFIDENCE.NORM, CONFIDENCE.T, COVARIANCE.P, COVARIANCE.S, DEVSQ, EXPON.DIST, F.DIST, F.DIST.RT, F.INV, F.INV.RT, F.TEST, FISHER, FISHERINV, FORECAST, FORECAST.LINEAR, FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY, FORECAST.ETS.STAT, GAMMA, GAMMA.DIST, GAMMA.INV, GAMMALN, GAMMALN.PRECISE, GAUSS, HARMEAN, HYPGEOM.DIST, KURT, LARGE, LINEST, LOGEST, LOGNORM.DIST, LOGNORM.INV, MAXA, MEDIAN, MINA, MODE.MULT, MODE.SNGL, NEGBINOM.DIST, NORM.DIST, NORM.INV, NORM.S.DIST, NORM.S.INV, PEARSON, PERCENTILE.EXC, PERCENTILE.INC, PERCENTRANK.EXC, PERCENTRANK.INC, PERMUT, PERMUTATIONA, PHI, POISSON.DIST, PROB, QUARTILE.EXC, QUARTILE.INC, RANK.AVG, RANK.EQ, SKEW, SKEW.P, SMALL, STANDARDIZE, STDEV, STDEV.P, STDEV.S, STDEVA, STDEVPA, T.DIST, T.DIST.2T, T.DIST.RT, T.INV, T.INV.2T, T.TEST, TREND, TRIMMEAN, VAR.P, VAR.S, VARPA, VARA, WEIBULL.DIST, Z.TEST

**Compatibility — wrappers (6)**: BETADIST, HYPGEOMDIST, LOGNORMDIST, NEGBINOMDIST, NORMSDIST, TDIST

**Compatibility — aliases (31)**: BETAINV, BINOMDIST, CHIDIST, CHIINV, CHITEST, CONFIDENCE, COVAR, CRITBINOM, EXPONDIST, FDIST, FINV, FTEST, GAMMADIST, GAMMAINV, LOGINV, MODE, NORMDIST, NORMINV, NORMSINV, PERCENTILE, PERCENTRANK, POISSON, QUARTILE, RANK, STDEVP, TINV, TTEST, VAR, VARP, WEIBULL, ZTEST

**Date and time (25)**: DATE, DATEDIF, DATEVALUE, DAY, EDATE, EOMONTH, MONTH, TIME, TIMEVALUE, HOUR, MINUTE, SECOND, NOW, TODAY, YEAR, NETWORKDAYS, NETWORKDAYS.INTL, DAYS, DAYS360, WEEKDAY, WEEKNUM, WORKDAY, WORKDAY.INTL, YEARFRAC, ISOWEEKNUM

**Financial (55)**: ACCRINT, ACCRINTM, DISC, FVSCHEDULE, INTRATE, PRICEDISC, PRICEMAT, RECEIVED, YIELDDISC, YIELDMAT, CUMIPMT, CUMPRINC, DB, DDB, DOLLARDE, DOLLARFR, EFFECT, FV, IPMT, IRR, ISPMT, MIRR, NOMINAL, NPER, NPV, PDURATION, PMT, PPMT, PV, RATE, RRI, SLN, SYD, TBILLEQ, TBILLPRICE, TBILLYIELD, XIRR, XNPV, DURATION, MDURATION, PRICE, YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD, COUPDAYBS, COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM, COUPPCD, AMORDEGRC, AMORLINC, VDB

**Engineering: Bessel and transcendental functions (8)**: BESSELI, BESSELJ, BESSELK, BESSELY, ERF, ERFC, ERFC.PRECISE, ERF.PRECISE

**Engineering: Number systems (12)**: BIN2DEC, BIN2HEX, BIN2OCT, DEC2BIN, DEC2HEX, DEC2OCT, HEX2BIN, HEX2DEC, HEX2OCT, OCT2BIN, OCT2DEC, OCT2HEX

**Engineering: Bit functions (5)**: BITAND, BITLSHIFT, BITOR, BITRSHIFT, BITXOR

**Engineering: Complex functions (26)**: COMPLEX, IMABS, IMAGINARY, IMARGUMENT, IMCONJUGATE, IMCOS, IMCOSH, IMCOT, IMCSC, IMCSCH, IMDIV, IMEXP, IMLN, IMLOG10, IMLOG2, IMPOWER, IMPRODUCT, IMREAL, IMSEC, IMSECH, IMSIN, IMSINH, IMSQRT, IMSUB, IMSUM, IMTAN

**Engineering: Misc function (4)**: CONVERT, DELTA, GESTEP, SUBTOTAL

**Database (17)**: DAVERAGE, DCOUNT, DGET, DMAX, DMIN, DSUM, DCOUNTA, DPRODUCT, DSTDEV, DVAR, DVARP, DSTDEVP, CORREL, RSQ, INTERCEPT, SLOPE, STEYX

