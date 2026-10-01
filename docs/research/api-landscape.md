# RustSheet: API Landscape and API Proposal

> Research snapshot produced at project inception (2026-10-01) by research agents. Treat as background, not as normative spec; normative docs live in `docs/architecture/` and `docs/process/`. Update via a PR when findings change.

Research date: 2026-10-01. The base for RustSheet is a fork of IronCalc (local clone at `scratchpad/IronCalc`, HEAD `e29e7b5`, 2026-09-30, `ironcalc_base` 0.8.3).

Scope: (1) the GRID Spreadsheet Engine API, (2) a survey of open-source engines, (3) a cross-engine comparison of API concepts, and (4) an API proposal for RustSheet. GRID is used as inspiration only. Nothing in section 4 copies GRID names or shapes verbatim where a better Rust-native design exists.

---

## 1. GRID Spreadsheet Engine

Sources: https://docs.grid.is/spreadsheet-engine/ , https://docs.grid.is/spreadsheet-engine/api/classes/model/ , https://docs.grid.is/spreadsheet-engine/api/classes/workbook/ , https://docs.grid.is/spreadsheet-engine/api/classes/worksheet/ , https://docs.grid.is/spreadsheet-engine/api/classes/cell/ , https://docs.grid.is/spreadsheet-engine/api/classes/range/ , https://docs.grid.is/spreadsheet-engine/api/classes/reference/ , https://docs.grid.is/spreadsheet-engine/tips/ , https://grid.is/engine . (`https://sdk.grid.is/` now 301-redirects to `https://docs.grid.is/`.)

### 1.1 Product and packaging
- npm `@grid-is/spreadsheet-engine`. The internal name is "Apiary" (docs at https://docs.grid.is/apiary/tutorials/quickstart/ and the `@grid-is/apiary` package). Written in TypeScript; the formula parser is a WASM module, so callers must `await Model.preconditions` first.
- Sibling packages: `@grid-is/spreadsheet-viewer`, `@grid-is/spreadsheet-editor` and `@grid-is/agent-tools` (LLM/MCP tooling).
- **License**: an evaluation license "for internal evaluation, prototypes, and proofs of concept only". Production use needs a commercial agreement, so GRID is not open source.
- Positioning: "deterministic recalculation, Excel and Google Sheets compatibility", headless in browser, Node, on-device or server. It is aimed at AI products (https://grid.is/ai-integration).

### 1.2 Object model (concrete classes)
| Class | Role |
|---|---|
| `Model` | Root container. It can hold **several workbooks** and resolves external references between them. It is the main read/write/recalc entry point and an event emitter. |
| `Workbook` | One workbook: sheets, defined names, tables (`TableManager`), pivots, styles, theme, comments/notes, views, `calcMode`. Structural edits live here. |
| `WorkSheet` | A sheet: cells, merges, row/column sizes, locally scoped names, spill lookup, navigation (`nextValueCellByID`, `nextBoundaryByID`). |
| `Cell` | A cell record with short JSF-style fields: `v` (value), `f` (formula), `F` (array-formula range), `ft` (`"a"` = array formula), `z` (number format), `M` (merge anchor), `style`, `href`, plus spill helpers `isSpillAnchor()`, `isSpilled()`, `spillWidth()`, `spillHeight()`. |
| `Range` | A pure rectangle with 0-based `top/left/bottom/right`, `$`-lock flags, and geometry ops (`intersects`, `getIntersection`, `except`, `moveBy`, `collapseTo*`, `iterCoordinates`). |
| `Reference` | A **resolvable** reference: workbook/sheet prefix plus range, or a defined name. It has flags `dynamic`, `conditional`, `trim` (trim-ref) and resolution methods `resolveCell`, `resolveAreaValues`, `resolveAreaCells`, `resolveAreaBoxed`. |
| `Matrix` | Array value (formula results, spills). It is stored compactly as four "quadrants": a dense top-left block, plus default row/column/value for the rest. A whole-column result such as `=A:A*2` therefore does not need a million entries. Has `map`, `hstack`, `permuteRows`, `iterPopulated`, and so on. |
| `Lambda` | A first-class lambda value: `parameterNames`, `expression` (AST), `boundArgs` (closures), `call(ctx, args)`. `TYPE_LAMBDA` is a value type, and `ArrayValue = CellValue \| Lambda`, so arrays can contain lambdas. |
| `DefinedName` | Implements the same `CellInterface` as `Cell`: it has `f`, `v` and number format, which effectively makes it a "formula cell without a position". |
| `Table`, `TableManager`, `PivotTable`, `PivotManager` | Structured objects. `Table` has `column(name)`, `rowRef(i)`, `appendRow`, `insertRow`, `rename` (rewrites structured references). |
| `ValueBox` | A value plus metadata, mainly a **formula-assigned number format**. For example, `=A1*2` inherits A1's currency format. Most read APIs come in plain and `*Boxed` variants. |
| `ValueSnapshot` | Captures every cell/spill/name value and later `applyTo(model)` restores it. This is GRID's what-if/undo substitute for value edits. |
| `VertexId` (`CellVertexId`, `RangeVertexId`, `NameVertexId`), `VertexIdSet` | Stable keys for nodes of the **dependency graph**. A `key` string is unique per model. A vertex is `(workbookKey, sheetIndex, row, col)` for cells, a rectangle for ranges, or `(workbookKey, sheetIndex\|null, name)` for names. Supports `contains`, `overlaps`, `offset`, `visitCells`. `VertexIdSet` de-duplicates equivalent ids. `Model.volatiles` returns vertex ids, and `ModelError.vertexIds` points at the cells involved. |
| Errors | `FormulaError` (an Excel error **value**), `ModelError` (diagnostic), `FormulaSyntaxError`, `EvaluationError`, `EditBlockedError` and 10 subclasses, `SheetNotFoundError`, `WorkbookNotFoundError`, `NameNotFoundError`. |

### 1.3 Values
- `CellValue = string | number | boolean | FormulaError | null`. `CellValueAtom` is the same type without errors or null.
- `ArrayValue = CellValue | Lambda`. `FormulaValue = ArrayValue | Reference | Matrix`, which means **a formula can evaluate to a reference**, not just a value. `FormulaArgument` adds `undefined` to mean "missing argument".
- Type tags are exported as constants: `TYPE_NUM, TYPE_STRING, TYPE_BOOL, TYPE_ERROR, TYPE_BLANK, TYPE_MISSING, TYPE_ARRAY, TYPE_RANGE, TYPE_LAMBDA`.
- `FormulaError` has `code` (as returned by `ERROR.TYPE`), `name` (`"#VALUE!"`), **`detail`** (for example `"YEARFRAC: basis must be 0, 1, 2, 3, or 4"`) and **`origin`**, a reference to the cell or name where the error *first* arose. The origin is preserved as the error propagates. Errors are memoized singletons (`ERROR_NA`, `ERROR_SPILL`, `ERROR_CALC`, `ERROR_FIELD`, `ERROR_GETDATA`, and so on), and `.detailed(text)` derives a variant with detail text.
- Mode bits `MODE_EXCEL`, `MODE_GOOGLE`, `MODE_GRID_SHEET`, `MODE_CANVAS` switch dialect semantics. `coerceNullToZero` is configurable.

### 1.4 Reads
- String-expression based: `model.readValue("Sheet2!B3")`, `readCell(expr)`, `readCells(expr, {cropTo})` (2-D `AreaCellArray`), and `readCellOrDefinedName`.
- `evaluateExpression(expr, {values, single, fallBack, cropTo, definedName, extraContext})` is the generic version. The `values` and `single` type parameters select between value, `Cell`, and a 2-D array of values or cells.
- `getCell(id, sheet?, workbook?)` and `workbook.getCell`, `sheet.getCellByID`, `getCells()`, `iterFormulaCells()`.
- Reads do **not** trigger computation. Values are whatever the last `recalculate` produced, apart from deferred cells (see `staleDeferredCells`).

### 1.5 Ad-hoc formulas
- `model.runFormula(formula, extraContext?) -> FormulaValue` evaluates a formula without writing it to a cell. The leading `=` is optional, results are **cached by formula text**, and it throws `FormulaSyntaxError` or `EvaluationError`. `extraContext` is a partial `EvaluationContext` (sheet name, cell, env, and so on).
- Related helpers: `analyzeAndFixFormula(formula)` (diagnose and propose fixes), `rewriteFormulaAfterMove(formula, from, to)`, `goalSeek(control, target, value)`, and the exported `parseFormula` and `functionSignatures`.

### 1.6 Writes
- `model.write(ref, value, opts?)` and `model.writeMultiple([[ref, value], ...], opts?)`. Both are **persistent** edits that recalculate automatically afterwards. `WriteOptions` has `skipRecalc`, `forceRecalc` and `skipVolatiles`. Per the docs, writes "have no transient mode that can be reverted"; to undo, capture a `ValueSnapshot` first.
- `workbook.editCell(ref, cellData)` writes value, formula and style together and updates graph edges. Calling `Cell.edit` directly is deprecated because it bypasses the graph.
- `clearCells(ref)`, `moveCells(from, to)`, `fill(source, dest, type)` (fill-handle semantics), `sortCells`, `mergeCells`/`unmergeCells`, `setRowHeight`/`setColumnWidth`.
- **Structural edits** (on `Workbook`): `insertRows(sheet, index, count, below)`, `deleteRows`, `insertColumns(..., toTheRight)`, `deleteColumns`, `insertCells(range, shift)`, `deleteCells(range, shift)`, `reorderRows`, `reorderColumns`, `moveCells`. Each returns a **`RewriteFormula` closure** `(formula, formulaSheet, formulaWorkbook) => string`. The caller applies it to *other* workbooks that reference the edited one, typically through `workbook.rewriteFormulas(fn)`.
- **Edit guards**: operations that Excel refuses throw a subclass of `EditBlockedError`. Examples are writing into pivot output, a data-table result, a non-anchor merged cell, or a table header; sorting across merges or spills; and pushing content off the sheet edge. The class names are `PivotEditBlockedError`, `MergedCellWriteBlockedError`, `TableEditBlockedError`, `SortCellsBlockedError`, `SheetEdgeBlockedError`, `DataTableEditBlockedError`, and so on, each with a `sheetName` and typed reason enums such as `SheetEdgeBlockedReason`. The docs make the philosophy explicit: Excel shows a modal or silently does nothing, while the engine throws, "so the host can decide".
- Defined names: `setDefinedName(name, formula, sheet?, abortOnError?, validateNow?)`, `removeDefinedName`, `getGlobal`, `isGlobal`. Table names share the defined-name namespace.

### 1.7 Recalculation and the dependency graph
- Each workbook keeps its **own dependency graph** keyed by vertex ids (cells, ranges, names). `recalculate(which)` runs one of three strategies, `CHANGED_OR_VOLATILE` (default), `CHANGED_ONLY` or `ALL_FORMULA_CELLS`, plus `includeDeferred`. It returns `{ nativeWorkbooksChanged }`.
- `calcMode`: `'auto' | 'autoNoTable' | 'manual'`, per workbook. It round-trips to xlsx, and the model-level value is derived from the workbooks. Data tables can be **deferred** (`deferDataTables`, `cellsToDefer`, `staleDeferredCells`), which is effectively a partial lazy mode for expensive what-if tables.
- Iterative calculation settings are available (`iterativeCalculationSettings()`), and cycles are reported (`RecalcCycleEvent`).
- Load options `AddWorkbookOptions` include `recalcAll`, `recalcErrors`, `recalcVolatiles`, `readOnly`, `isExternal`, `iterativeCalculation`, `extractExpressions` (an optimizer that hoists repeated subexpressions into internal defined names) and `assumeArrayFormulas`.
- `model.volatiles` lists vertex ids of volatile cells and names.

### 1.8 Events
`Model` is an emitter with `on`, `off`, `once`, `emit` and `listeners`. The typed event map is `ModelEventArgs`:
- `beforerecalc` and `recalc` (payload: the model).
- `recalcEvent` is a **per-cell trace** during recalculation. Its kinds are `Trigger`, `Root`, `MarkDirty`, `EnqueueMaybe`, `Propagate`, `Evaluate`, `Unreached`, `Reorder` and `Cycle`.
- `error` carries a `ModelError` with `seenBefore` for de-duplication, plus workbook id and name.
- `metrics` gives per-function timing samples.
- `addsheet`, `attach`, `detach` and `native-update` (a host-forwarded invalidation).

There is **no "which cells changed" payload** on `recalc`. A host has to diff values itself or listen to `recalcEvent`.

### 1.9 Diagnostics
`ModelError` has a `level` (`NONE`, `NOTICE`, `WARNING`, `ERROR`), a `type` string, `vertexIds`/`references` (the cells involved) and `origException`. `model.errors` and `workbook.errors` accumulate them, separately from Excel error values.

### 1.10 Serialization
`Model.fromXLSX`, `fromXLSXFile`, `fromJSF` and `fromCsf`, plus `toXLSX`, `toXLSXFile`, `toJSF` and `toCSF`. `serializeModel(model) -> Buffer` and `deserializeModel(buf)` produce a full binary snapshot; GRID recommends this for reverts across structural edits.

### 1.11 Design philosophy (as inferred)
- **Mutable, imperative model.** It is not immutable, and there are no transactions and no undo stack. Batching is `writeMultiple` plus `skipRecalc`.
- **Revert by snapshot.** `ValueSnapshot` restores values only, best effort, with explicit warnings when formulas changed. `serializeModel` gives full fidelity.
- **Excel-faithful guards** that throw typed errors instead of silently doing nothing.
- **String-first addressing** (`"Sheet1!A1"`, names, structured refs) everywhere, with `Reference`/`Range` objects as the typed alternative.
- **Rich error values** carrying detail and origin, which helps debugging and LLM explanations.
- **Compact array and number-format propagation** (`Matrix` quadrants, `ValueBox`) for fidelity with Excel's displayed results.
- **Multi-workbook model** with external links resolved live.
- **Observability** through fine-grained recalc events and metrics.

---

## 2. Survey of engines

Star counts are from the GitHub pages fetched in October 2026 where stated; "n/f" means not fetched.

| Engine | Lang | License | Maturity / activity | Fidelity claims | Architecture | API style |
|---|---|---|---|---|---|---|
| **IronCalc** https://github.com/ironcalc/IronCalc | Rust (+WASM, Python, Node bindings) | MIT OR Apache-2.0 | ~4.2k stars. v0.8 (Jan 2026) added dynamic arrays, spill, LAMBDA/LET, CF, themes and 100+ functions. The README still says "work-in-progress". NLnet funded. Active, with daily commits. | ~496 entries in the `Function` enum (counted locally). The roadmap targets 90% of Excel functions. | **No persistent dependency graph.** `Model::evaluate()` re-evaluates the **whole workbook** by top-down recursion. Dynamic-array anchors are ordered by learned "restart" facts, and depth/stack-bounded unwinding avoids stack overflow (`base/src/evaluation.rs`). Shared formulas, a shared-string table, and an xlsx crate for IO. | Index-based imperative methods: `Model::set_user_input(sheet, row, col, String)`, `get_cell_value_by_index`, `evaluate()`. `UserModel` adds undo/redo, a diff queue for collaboration and `pause_evaluation`. Errors are `Result<_, String>`. |
| **Formualizer** https://github.com/psu3d0/formualizer | Rust (+Python, WASM, C FFI) | MIT OR Apache-2.0 | ~184 stars, v0.10 (2026). Young but very active. Has a `public-api/` snapshot for semver discipline. | "400+ functions", edge cases checked against Excel; dynamic arrays, LET, LAMBDA. | Arrow-column storage. **Formula families**: a copied run is one template and one graph node. Region-based dependency tracking, **incremental recalc**, recalc plans, parallel evaluation, cancellation and budgets, deterministic mode (clock, tz, seed), transactional change log with undo. Claims 1M formulas in 0.38 s and a 52 ms incremental recalc. | `Workbook::set_value(sheet, row, col, LiteralValue)`, `set_formula`, `evaluate_cell`, `evaluate_all`, `*_with_delta` (returns an `EvalDelta`), `begin_action`/`end_action`, `Engine::precedents`/`dependents`, `insert_rows -> ShiftSummary`, custom functions, WASM plugins. "SheetPort" exposes a workbook as a typed function. |
| **HyperFormula** https://github.com/handsontable/hyperformula | TypeScript | **GPLv3** or commercial (AGPLv3 until 3.0, Jan 2025) | ~2.8k stars, 3.x, maintained by Handsontable. | ~400 functions (423 listed). **No LAMBDA/LET** (discussion #1268). Array functions and `#SPILL!` exist. | Dependency graph with incremental recalc, address mapping strategies, lazy CRUD transformations, undo/redo, clipboard, i18n (17 languages). | `buildFromArray`/`buildFromSheets`, `{sheet,row,col}` addresses, `setCellContents(addr, content) -> ExportedChange[]`, `batch(fn)`, `suspendEvaluation`/`resumeEvaluation`, `isItPossibleTo*` preflight, `calculateFormula(formula, sheetId)`, `getCellDependents`/`getCellPrecedents`, events `valuesUpdated`, `sheetAdded`, and so on. |
| **Univer** https://github.com/dream-num/univer | TypeScript | Apache-2.0 (some "pro" features are commercial) | ~22k stars, 1.0 released, very active. A full office SDK. | "Hundreds" of Excel functions, dynamic arrays. | Separate `@univerjs/engine-formula`: AST plus dependency graph, async calculation (can run in a web worker), command and mutation architecture (good for collaboration). Headless in Node. | "Facade API": `univerAPI.getActiveWorkbook().getSheetByName(..).getRange('A1').setValue(..)`. `FFormula`: `executeCalculation()`, `calculationEnd(cb)`, `onCalculationEnd()`. Heavy DI framework, not a minimal engine. |
| **LibreOffice Calc** https://git.libreoffice.org/core (`sc/`) | C++ | MPL-2.0 | 25+ years old, the most complete open engine. | Very high ODF/OOXML fidelity, but not 100% Excel (differences in dynamic arrays, some functions). | Broadcaster/listener dirty tracking (`ScFormulaCell` listens, then goes on the FormulaTrack list). **Formula groups** are calculated as a unit via threads or OpenCL ("software interpreter" as fallback). | UNO API (`XCell.setFormula`, `XCalculatable.calculate`), heavy process, usually driven headless via soffice. |
| **Apache POI** https://poi.apache.org/components/spreadsheet/eval.html | Java | Apache-2.0 | Mature, slow-moving. | A large subset of Excel functions (count not verified here), no dynamic arrays or LAMBDA. | Evaluator with a cell-value **cache** that the caller must invalidate manually. | `FormulaEvaluator.evaluate(cell)`, `evaluateFormulaCell`, `evaluateAll()`, `notifyUpdateCell(cell)`/`notifySetFormula`/`clearAllCachedResultValues()`. |
| **pycel** https://github.com/dgorissen/pycel | Python | GPL-3.0 | ~630 stars, low activity. | "Functions the developer requires". | Compiles Excel formulas to Python; graph plus lazy evaluation with caching. | `ExcelCompiler(filename=...)`, `.evaluate('Sheet!A1')`, `.set_value(addr, v)`, `.validate_calcs()`. |
| **formulas** https://github.com/vinci1it2000/formulas | Python | EUPL-1.1+ | ~500 stars, 1.3.4 (Mar 2026). | 483/536 functions (90.1%), circular references via `finish(circular=True)`. | Builds a `schedula` dispatcher DAG; can **compile a sub-model** into a function of chosen inputs and outputs. | `ExcelModel().loads(path).finish()`, `.calculate()`, `.compile(inputs=[...], outputs=[...])(*args)`. |
| **xlcalculator** https://github.com/bradbase/xlcalculator | Python | MIT | ~160 stars, modernised koala2. | ~50 to 100 functions, no array or CSE formulas. | Model compiled to an evaluator graph. | `ModelCompiler().read_and_parse_archive(path)`, `Evaluator(model).set_cell_value(addr, v)`, `.evaluate(addr)`. |
| **EtherCalc / SocialCalc** https://github.com/audreyt/ethercalc | JS / Node | CPAL-1.0 (EtherCalc), Artistic-2.0 (SocialCalc) | Effectively dormant. | Low, with its own function set. | Command-log based multi-user sheet; recalc is full-sheet. | Text command protocol (`set A1 value n 3`), REST. Historical interest only (command-log collaboration). |
| **Microsoft Excel JS API** https://learn.microsoft.com/en-us/javascript/api/excel/excel.range | JS (Office Add-ins) | Proprietary (reference API, not an engine) | Mature, versioned requirement sets. | It *is* Excel. | Remote proxy objects. | `Excel.run(ctx => ...)`, proxy objects with `load()` plus `context.sync()` batching, `Range.values`/`formulas`/`valueTypes`/`valuesAsJson` (1.16, rich error subtypes), `Range.insert(shift)`/`delete(shift)`, `getDirectPrecedents` (1.12), `getPrecedents` (1.14), `getDirectDependents` (1.13), `getDependents` (1.15), `getSpillingToRange`/`getSpillParent` (1.12), `Application.calculationMode`, `calculate()`, `Worksheet.enableCalculation`, events `onChanged`, `onCalculated`, `onFormulaChanged`, `onRowSorted`, `onFiltered`, `onCalculationBusy`, and more. |
| **umya-spreadsheet** https://github.com/MathNya/umya-spreadsheet | Rust | MIT | Active. | Not applicable (no calculation). | Full read-modify-write xlsx object model with styles, charts and so on. Formulas are stored as strings only. | `reader::xlsx::read`, `get_sheet_by_name_mut(..).get_cell_mut("A1").set_value(..)`, `writer::xlsx::write`. Candidate for **round-trip IO** of parts that IronCalc drops. |
| **calamine** https://github.com/tafia/calamine | Rust | MIT | Very active, widely used. | Not applicable (read only). | Fast streaming reader for xlsx, xlsm, xlsb, xls and ods. Returns cached values and formula strings. | `open_workbook_auto(path)?.worksheet_range("Sheet1")`, `worksheet_formula`. Good for **fast bulk load**. Formualizer uses it as a reader backend. |
| **rust_xlsxwriter** https://github.com/jmcnamara/rust_xlsxwriter | Rust | MIT OR Apache-2.0 | Very active (port of Python XlsxWriter). | Not applicable (write only). | High-fidelity xlsx writer: charts, tables, CF, data validation, dynamic-array formulas, and so on. | `Workbook::new().add_worksheet().write(row, col, ..)`, `save(path)`. Good for **export** of rich features. |

Commercial reference points (not surveyed in depth): SpreadsheetGear (.NET), Aspose.Cells (Java/.NET), MESCIUS SpreadJS calc engine, and Microsoft Graph's Excel REST API (the workbook session model).

Takeaways for RustSheet:
1. **IronCalc** gives a strong permissive base: fidelity work, dynamic arrays, LAMBDA, xlsx IO, locales and i18n. Its weaknesses are API design and incremental recalc. There is no dependency graph, every evaluation is a full recalc, error values come back as localized strings, the error type is `String`, coordinates are 1-based `i32` with the sheet as a positional index, and `Model<'a>` carries a lifetime tied to locale data (the Python binding has to `leak_str`).
2. **Formualizer** is the closest open competitor in Rust and shows what "fast" looks like: formula families, Arrow, region-based dependencies, deltas, determinism, cancellation. RustSheet should match those *concepts* while keeping a smaller, cleaner public surface (Formualizer's `Workbook` exposes about 90 methods, with many `*_with_delta_policy_cancellable` permutations).
3. **HyperFormula** gives the best ergonomic ideas for change sets (`ExportedChange[]` returned from every mutation), preflight checks (`isItPossibleTo*`) and `batch`. Its GPL license rules out reusing code.
4. **GRID** gives the best ideas on rich error values (detail and origin), typed edit-guard errors, multi-workbook external links, compact matrices, number-format propagation, and observability.
5. **Excel JS** sets the vocabulary users expect: Range with `values`, `formulas`, `valueTypes`; `insert`/`delete` with `shift`; direct versus transitive precedents and dependents; calculation modes; change events with before/after.

---

## 3. Comparison of API concepts

| Concept | GRID | IronCalc (base/src/model.rs, user_model/) | HyperFormula | Formualizer | Excel JS API |
|---|---|---|---|---|---|
| Root object | `Model` (several workbooks) | `Model<'a>` (raw), `UserModel<'a>` (undo, diffs) | `HyperFormula` instance | `Workbook` wrapping `Engine<R>` | `Excel.run` context, then `context.workbook` |
| Cell addressing | Strings `"Sheet1!A1"`, `Reference`, 0-based `Range` | `(sheet: u32 index, row: i32, col: i32)`, **1-based**; `get_cell_value_by_ref("Sheet1!A1")` | `{sheet: id, row, col}`, 0-based | `(&str sheet, row: u32, col: u32)`, 1-based, plus `RangeAddress` | `getRange("A1:B2")`, `getCell(r,c)` 0-based |
| Sheet identity | name or index | positional index (plus internal `sheet_id`) | numeric sheet id | name (plus internal `SheetId`) | `Worksheet` object (id, name) |
| Value type | `string\|number\|boolean\|FormulaError\|null`, plus `Matrix`, `Reference`, `Lambda` | `CellValue{None,String,Number,Boolean}`; **errors come back as `String`** | `CellValue` incl. `DetailedCellError` | `LiteralValue{Int,Number,Text,Boolean,Error,Array,Date,DateTime,Time,Duration,...}` | `any[][]` plus `valueTypes`, `valuesAsJson` (typed, with error subtypes) |
| Error value detail | `detail` plus `origin` ref | internally `CalcResult::Error{error, origin, message}`, flattened to a string at the API | `type`, `message`, `address` | `ExcelError{kind, message, ...}` (`Circ`, `Cancelled`, `NImpl`, ...) | `valuesAsJson` error subtype |
| API error type | JS exceptions (typed classes) | `Result<_, String>` | exceptions (typed) | `ExcelError` / `IoError` / `EditorError` | `OfficeExtension.Error` with code |
| Single write | `model.write(ref, v)` | `set_user_input(s,r,c,String)`, `update_cell_with_number/text/bool/formula` | `setCellContents(addr, content)` | `set_value`, `set_formula` | `range.values = [[..]]`, `range.formulas = ..` |
| Batch write | `writeMultiple`, `skipRecalc` | `UserModel::pause_evaluation()`/`resume_evaluation()` | `batch(fn)`, `suspendEvaluation`/`resumeEvaluation` | `set_values`, `write_range`, `begin_action`/`end_action`, `Engine::begin_batch` | queued commands until `sync()` |
| Atomic rollback | No (use `ValueSnapshot`/`serializeModel`) | No (undo via `UserModel`) | No (undo exists) | change log with rollback | No |
| Mutation returns | `void` or `RewriteFormula` | `()` | `ExportedChange[]` (changed values) | `()`; the delta comes from `evaluate_*_with_delta` | proxy; events carry details |
| Recalc engine | dependency graph, incremental | **full re-evaluation**, no graph | dependency graph, incremental | region graph, incremental, parallel, families | Excel |
| Calc modes | `auto`, `autoNoTable`, `manual`; deferred cells | auto (UserModel) or explicit `evaluate()` | auto / suspended | explicit `evaluate_*` (plus plans) | `automatic`, `automaticExceptTables`, `manual`; `enableCalculation` per sheet |
| Compute on read | No (except deferred) | No | No | `evaluate_cell(s)` evaluates the needed cone | No |
| Ad-hoc formula | `runFormula(f, ctx)` (cached), `evaluateExpression` | **none public** | `calculateFormula(f, sheetId)` | via engine (not first-class) | `Application`: none (needs a cell) |
| Precedents / dependents | internal graph, `VertexId`s (no public query documented) | none | `getCellPrecedents`/`getCellDependents` (direct) | `Engine::precedents`/`dependents` with options | direct and transitive variants |
| Structural edits | `insertRows`, `deleteRows`, `insertCells(range, shift)`, `moveCells`, `reorderRows`, ...; return `RewriteFormula` | `insert_rows`, `delete_rows`, `insert_columns`, `delete_columns`, `move_rows_action`, `move_columns_action` | `addRows(sheet, [i, n]...)`, `removeRows`, `moveRows`, `setRowOrder`, plus `isItPossibleTo*` | `insert_rows`/`delete_rows`/... returning `ShiftSummary` | `range.insert(shift)`, `range.delete(shift)` |
| Edit guards | `EditBlockedError` family | ad-hoc `Err(String)` | `isItPossibleTo*` plus exceptions | `EditorError` | `InvalidOperation` errors |
| Defined names | `setDefinedName(name, formula, sheet?)` | `new_defined_name(name, scope: Option<u32>, formula)`, `update_`, `delete_` | `addNamedExpression(name, expr, scope)` | `define_named_range(name, RangeAddress, scope)` | `names.add(name, ref)` |
| Tables | `TableManager`, `Table` (column, rowRef, appendRow) | imported from xlsx; no creation API | none | `define_table`, `table_metadata` | `tables.add`, `TableRowCollection` |
| Dynamic arrays / spill | yes (`Matrix`, `isSpillAnchor`) | yes (v0.8) | partial (`#SPILL!`, array functions) | yes | yes (`getSpillingToRange`) |
| LAMBDA / LET | yes (first-class `Lambda` value) | yes | **no** | yes | yes |
| Events | emitter: `recalc`, `recalcEvent` (per cell), `error`, `metrics`, `addsheet` | none (diff queue `flush_send_queue`) | `valuesUpdated`, `sheet*`, `namedExpression*`, `evaluation*` | none (pull deltas, change log) | `onChanged`, `onCalculated`, `onFormulaChanged`, ... |
| Undo | No (snapshots) | `UserModel::undo`/`redo` | `undo`/`redo` | `undo`/`redo` via change log | Ctrl+Z only (not API) |
| Snapshot / serialize | `serializeModel`, `ValueSnapshot` | `to_bytes()` (bitcode `.icalc`) | `getAllSheetsSerialized` | JSON workbook format | not applicable |
| Multi-workbook | yes (external refs live) | no | no | no | not applicable |
| Determinism controls | `env` map | `mock_time` feature (tests) | config | `DeterministicMode` (clock, tz, seed) | no |
| Threading | single-threaded JS | `&mut self` everywhere; lifetime `'a` | single-threaded JS | `&mut`; parallel eval inside | not applicable |
| IO | xlsx, JSF, CSF | xlsx, `.icalc` | arrays only (no xlsx) | xlsx (calamine, umya), CSV, JSON | not applicable |
| Bindings | JS/TS | Rust, Python (PyO3), WASM, Node | JS | Rust, Python, WASM, C | JS |
| License | evaluation / commercial | MIT/Apache-2.0 | GPLv3 / commercial | MIT/Apache-2.0 | proprietary |

---

## 4. RustSheet API proposal

### 4.1 Design principles
1. **Values are data; failures are `Result`.** Excel errors (`#N/A`, `#SPILL!`, ...) are ordinary `Value::Error`s and never Rust `Err`s. `Err` is reserved for API misuse or a refused operation: bad reference, unknown sheet, parse error, edit blocked, cancelled.
2. **Every mutation goes through a transaction and returns a `ChangeSet`.** One code path handles single edits, batches, undo, events and collaboration diffs. Transactions are atomic: an error rolls everything back.
3. **Stable identities.** `SheetId` survives rename and reorder. Coordinates are 0-based `u32` internally. A1 strings are an input convenience that is parsed once.
4. **Snapshots are cheap, immutable and `Send + Sync`.** Reads are possible from many threads while one writer works. What-if scenarios fork without copying the workbook.
5. **Lazy by default where safe.** The engine computes only what someone observes (watched ranges, explicit reads, export). Eager automatic mode is still available for Excel parity.
6. **No lifetimes and no generics on the public surface.** Every public type is owned and `'static`, so it maps directly to PyO3, UniFFI, JNI and C.
7. **Small core, layered extras.** `rustsheet` (core), `rustsheet-xlsx` (IO), `rustsheet-py`, `rustsheet-ffi` (UniFFI and C ABI). UI concerns such as selection, autofill, clipboard and borders, which live in IronCalc's `UserModel`, stay out of the headless core.

### 4.2 Core types (Rust sketch)

```rust
// ---------- identities & references ----------
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)] pub struct SheetId(u32);   // stable across rename/move
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)] pub struct BookId(u32);    // for multi-workbook / external links

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CellRef { pub sheet: SheetId, pub row: u32, pub col: u32 }          // 0-based

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct RangeRef { pub sheet: SheetId, pub top: u32, pub left: u32, pub bottom: u32, pub right: u32 } // inclusive
// whole rows/cols use bottom = MAX_ROW / right = MAX_COL; helpers: RangeRef::rows(..), ::cols(..)

/// Anything a user can point at. Parsed from strings via `wb.reference("Sheet1!A1:B2")`.
#[non_exhaustive]
pub enum Ref {
    Cell(CellRef),
    Range(RangeRef),
    Name { name: Arc<str>, scope: Scope },
    TableColumn { table: Arc<str>, column: Arc<str>, part: TablePart },
    MultiArea(Vec<RangeRef>),                // (A1:B2,D4) unions
}

/// Everything that addresses cells implements this, so APIs take `impl IntoRef`
/// and accept "Sheet1!A1", CellRef, RangeRef, (SheetId,row,col), or a Name.
pub trait IntoRef { fn into_ref(self, ctx: &RefContext) -> Result<Ref, RefError>; }

// ---------- values ----------
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Value {
    Empty,
    Number(f64),
    Text(Arc<str>),
    Bool(bool),
    Error(XlError),
    Array(Arc<Array>),          // only as a formula/eval result; cells hold scalars (spills expose per-cell scalars)
}

#[derive(Clone, Debug, PartialEq)]
pub struct XlError {
    pub kind: ErrorKind,                 // Excel semantics + ERROR.TYPE code via kind.code()
    pub detail: Option<Arc<str>>,        // "YEARFRAC: basis must be 0..4"
    pub origin: Option<CellRef>,         // where it first arose (kept through propagation)
}
#[non_exhaustive]
pub enum ErrorKind { Null, Div0, Value, Ref, Name, Num, NA, GettingData, Spill, Calc, Field,
                     Blocked, Connect, Busy, Unknown, Circ /* engine-level */, NotImplemented }

/// 2-D array with GRID-style "default fill": dense populated block + default for the rest.
/// Whole-column results cost O(populated), not O(1M).
pub struct Array { rows: u32, cols: u32, dense: Vec<Value>, dense_rows: u32, dense_cols: u32, fill: Value }
impl Array { pub fn get(&self, r: u32, c: u32) -> &Value; pub fn rows(&self) -> u32; pub fn cols(&self) -> u32;
             pub fn iter_rows(&self) -> impl Iterator<Item = &[Value]>; }

// ---------- inputs ----------
pub enum Input {
    Value(Value),                 // literal, stored as-is ("=1" is TEXT)
    Formula(Arc<str>),            // "=SUM(A1:A3)" (leading '=' optional)
    ArrayFormula(Arc<str>, RangeRef),     // legacy CSE
    Parse(Arc<str>),              // Excel "typed into a cell" semantics: "12%", "1/2/2026", "=..", "'text"
    Clear,
}
impl From<f64> for Input { .. } impl From<&str> for Input /* = Parse */ { .. } impl From<bool> for Input { .. }

// ---------- reading cell details ----------
pub struct CellInfo {
    pub value: Value,
    pub formula: Option<Arc<str>>,           // in the workbook's formula locale
    pub number_format: Option<Arc<str>>,     // explicit
    pub effective_format: Option<Arc<str>>,  // incl. formula-inherited (GRID's "boxed" idea, as a field)
    pub spill: SpillRole,                    // None | Anchor{extent: RangeRef} | Member{anchor: CellRef} | Blocked
    pub stale: bool,                         // true if a lazy/manual model hasn't computed it yet
}
```

Notes:
- `Value::Array` appears only in evaluation results. Stored cells always hold scalars, so a spill anchor has `SpillRole::Anchor` and each member cell reads its own scalar. This avoids GRID's split between `Matrix` and `Cell` for consumers.
- There is no `Lambda` or `Reference` in the public `Value`. `eval` returns `EvalOutput { value, reference: Option<Ref> }`, so callers learn that `=OFFSET(..)` resolved to a range without a third value kind. A lambda result surfaces as `Value::Error(Calc)`, which is what Excel shows; internally the engine keeps a richer `CalcValue`.

### 4.3 Workbook, snapshots and reads

```rust
pub struct Workbook { /* single writer; Send, !Sync */ }
pub struct Snapshot { /* Arc-shared persistent state; Send + Sync + Clone, O(1) clone */ }

impl Workbook {
    pub fn new(opts: WorkbookOptions) -> Self;                  // locale, tz, language, calc mode, determinism
    pub fn sheet_id(&self, name: &str) -> Result<SheetId, RefError>;
    pub fn sheets(&self) -> impl Iterator<Item = SheetMeta> + '_;
    pub fn reference(&self, a1: &str) -> Result<Ref, RefError>;  // parse once, reuse

    // reads (never mutate; return last computed values; see CalcMode for freshness)
    pub fn value(&self, r: impl IntoRef) -> Result<Value, RefError>;               // single cell (top-left of range)
    pub fn values(&self, r: impl IntoRef) -> Result<Array, RefError>;              // 2-D
    pub fn cell(&self, r: impl IntoRef) -> Result<CellInfo, RefError>;
    pub fn formatted(&self, r: impl IntoRef) -> Result<String, RefError>;          // number format + locale
    pub fn used_range(&self, sheet: SheetId) -> Option<RangeRef>;
    pub fn iter_cells(&self, range: RangeRef, filter: CellFilter) -> impl Iterator<Item = (CellRef, CellInfo)> + '_;

    pub fn snapshot(&self) -> Snapshot;                          // O(1); readers on other threads
}
impl Snapshot { /* same read API as Workbook, plus lazy compute (4.5) */ }
```

Implementation note: sheets are stored in chunked, `Arc`-shared column blocks (for example 1,024-row chunks per column, copy-on-write via `Arc::make_mut`). A snapshot shares chunks, and a write copies only the touched chunk. This is what makes `snapshot()`, `fork()` and what-if cheap. Formualizer's Arrow columns are a comparable design point; RustSheet can adopt Arrow for numeric columns later without changing the API.

### 4.4 Writes: transactions and change sets

```rust
impl Workbook {
    /// Atomic edit. All-or-nothing: on Err every change is rolled back, and nothing is recalculated.
    pub fn edit<T>(&mut self, f: impl FnOnce(&mut Txn) -> Result<T, EditError>) -> Result<(T, ChangeSet), EditError>;

    /// Builder alternative (for bindings): drop without commit = rollback.
    pub fn begin(&mut self) -> Txn<'_>;

    /// Sugar for a one-cell transaction.
    pub fn set(&mut self, r: impl IntoRef, input: impl Into<Input>) -> Result<ChangeSet, EditError>;

    pub fn undo(&mut self) -> Result<ChangeSet, EditError>;    // ChangeSets carry their inverse
    pub fn redo(&mut self) -> Result<ChangeSet, EditError>;
}

impl Txn<'_> {
    pub fn set(&mut self, r: impl IntoRef, input: impl Into<Input>) -> Result<(), EditError>;
    pub fn set_values(&mut self, top_left: impl IntoRef, rows: &[&[Value]]) -> Result<(), EditError>;
    pub fn set_formulas_r1c1(&mut self, range: RangeRef, formula_r1c1: &str) -> Result<(), EditError>; // fill a family in O(1)
    pub fn clear(&mut self, r: impl IntoRef, what: ClearWhat) -> Result<(), EditError>;
    pub fn set_style(&mut self, r: impl IntoRef, style: StylePatch) -> Result<(), EditError>;
    // structural (4.7), names (4.8), tables (4.8), sheets: add_sheet / rename_sheet / move_sheet / delete_sheet
    pub fn label(&mut self, text: &str);                        // for undo history / audit
    pub fn check(&self, op: &Op) -> Result<(), EditError>;      // preflight without applying (HyperFormula's isItPossibleTo*)
    pub fn commit(self) -> Result<ChangeSet, EditError>;
}

pub struct ChangeSet {
    pub id: ChangeId,
    pub inputs: Vec<InputChange>,          // what the caller wrote: {cell, before: CellContent, after: CellContent}
    pub values: Vec<ValueChange>,          // what recalculation changed: {cell, before: Value, after: Value}
    pub spills: Vec<SpillChange>,          // anchors whose extent changed
    pub structure: Vec<StructuralChange>,  // insert/delete/move with a RefMap (4.7)
    pub names: Vec<NameChange>,
    pub stale: Option<StaleSet>,           // in Manual/Lazy mode: what is now dirty but not computed
    pub diagnostics: Vec<Diagnostic>,      // warnings (e.g., new circular ref)
}
```

Why this is better:
- **GRID**: `write` and `writeMultiple` have no rollback, no undo and no change list. `recalc` events do not say *what* changed. RustSheet returns before/after values for exactly the cells that changed, and `undo` is built in.
- **IronCalc**: `UserModel` has undo, but its diff list is an opaque bitcode blob for collaboration, and the raw `Model` has nothing. RustSheet gives one typed `ChangeSet` that is serializable with `serde`, and that same type serves undo, events, audit logs and collaborative sync.
- **HyperFormula**: `batch` returns changes but cannot roll back a half-failed batch. RustSheet's `edit` is atomic.
- **Formualizer**: deltas are only available from `evaluate_*_with_delta`, while `set_value` returns `()`. RustSheet ties the delta to the edit itself.
- Bulk writes stay fast because recalculation runs once at commit. `ChangeSetOptions { record_before: bool, max_value_changes: Option<usize> }` keeps very large commits cheap; past the limit `values` is truncated and `values_truncated = true`.

### 4.5 Recalculation: explicit, automatic and lazy

```rust
pub enum CalcMode {
    Automatic,                 // Excel default: commit recalcs everything dirty before returning
    AutomaticExceptTables,     // data tables (TABLE()) deferred (Excel/GRID parity)
    Manual,                    // commit marks dirty only; caller runs recalc()
    OnDemand,                  // commit marks dirty; reads compute exactly the cone they need (compute-on-read)
}

impl Workbook {
    pub fn set_calc_mode(&mut self, m: CalcMode);
    pub fn recalc(&mut self, scope: RecalcScope, opts: RecalcOptions) -> Result<ChangeSet, CalcError>;
    pub fn compute(&mut self, targets: &[Ref]) -> Result<Vec<Value>, CalcError>;   // evaluate only these cones
    pub fn watch(&mut self, r: impl IntoRef) -> WatchId;    // OnDemand: watched ranges are always kept fresh
    pub fn unwatch(&mut self, id: WatchId);
    pub fn is_stale(&self, r: impl IntoRef) -> bool;
}
pub enum RecalcScope { Dirty, DirtyAndVolatile, Full /* = Ctrl+Alt+F9 */, Sheet(SheetId), Targets(Vec<Ref>) }
pub struct RecalcOptions {
    pub cancel: Option<CancelToken>, pub deadline: Option<Instant>,
    pub include_deferred: bool, pub threads: Threads /* Auto | Single | N(usize) */,
    pub iteration: Option<IterationSettings>,   // max_iter, max_change (Excel iterative calc)
}
impl Snapshot {
    /// Compute-on-read with &self: a per-snapshot concurrent memo of dirty cells.
    /// Safe from many threads; results are cached in the snapshot, never written back.
    pub fn compute(&self, targets: &[Ref]) -> Result<Vec<Value>, CalcError>;
}
```

Engine plan, as the work to do on the IronCalc fork:
1. **Add a persistent dependency graph.** IronCalc has none today. Nodes are cells, formula families (runs of a shared formula), ranges (bucketed by row and column blocks, as an interval index) and names. Dirty propagation uses an explicit work-list, plus topological levels for parallel evaluation with rayon.
2. **Keep IronCalc's evaluator semantics** (dynamic-array anchor ordering, restart learning, stack-bounded recursion) for evaluating the *dirty subgraph*. Full recalculation remains the oracle in tests, which is the same approach Formualizer took.
3. Volatile functions, `INDIRECT` and `OFFSET` are handled as dynamic dependencies: the cell is recorded as volatile, and the dependencies actually read are captured during evaluation.
4. Determinism: `WorkbookOptions { clock: Arc<dyn Clock>, rng_seed: Option<u64>, tz }`. This generalizes IronCalc's `mock_time` feature into runtime injection.

Why this is better: GRID offers auto or manual plus deferred data tables, and its reads never compute. HyperFormula only has suspend and resume. IronCalc always runs a full evaluation. RustSheet's `OnDemand` mode lets a server load a 200k-formula model and answer "what is Summary!B5 if Inputs!B3 = 7%?" by evaluating only that cone. `Snapshot::compute` lets many threads do this concurrently over the same immutable model.

### 4.6 Ad-hoc evaluation, scenarios and "workbook as a function"

```rust
impl Workbook /* and Snapshot */ {
    pub fn eval(&self, formula: &str, ctx: EvalContext) -> Result<EvalOutput, ParseError>;
    pub fn compile(&self, formula: &str, ctx: EvalContext) -> Result<CompiledFormula, ParseError>; // reuse; tracks deps
}
pub struct EvalContext {
    pub sheet: Option<SheetId>,          // for unqualified refs
    pub anchor: Option<CellRef>,         // for relative refs / ROW() / implicit intersection
    pub bindings: Vec<(Arc<str>, Value)>,// host-supplied LET-style variables: eval("=rate*12", bind("rate", 0.05))
    pub array_mode: ArrayMode,           // Spill (dynamic) | ImplicitIntersection (legacy)
}
pub struct EvalOutput { pub value: Value, pub reference: Option<Ref> }

/// What-if without mutating the base: an overlay transaction evaluated on a snapshot.
impl Snapshot {
    pub fn with_inputs(&self, inputs: &[(Ref, Input)]) -> Result<Scenario, EditError>; // cheap fork
}
impl Scenario { pub fn value(&self, r: impl IntoRef) -> Result<Value, RefError>; /* lazy, cone-only */ }

/// Freeze a workbook into a pure function of chosen inputs → outputs (like `formulas.compile`, Formualizer SheetPort).
pub struct Calculator { /* Send + Sync */ }
impl Snapshot { pub fn calculator(&self, inputs: &[Ref], outputs: &[Ref]) -> Result<Calculator, CalcError>; }
impl Calculator {
    pub fn call(&self, args: &[Value]) -> Result<Vec<Value>, CalcError>;
    pub fn call_batch(&self, rows: &[Vec<Value>]) -> Result<Vec<Vec<Value>>, CalcError>; // parallel scenario sweep
}
```

Why this is better than GRID: `runFormula` hides caching behind formula text and has no host bindings, and GRID's what-if relies on mutate-then-restore through `ValueSnapshot`, which is "best-effort" per its own docs. RustSheet's scenarios are side-effect free, run in parallel and are correct by construction. IronCalc has no ad-hoc evaluation at all.

### 4.7 Structural edits with reference adjustment

```rust
impl Txn<'_> {
    pub fn insert_rows(&mut self, sheet: SheetId, at: u32, count: u32) -> Result<(), EditError>;
    pub fn delete_rows(&mut self, sheet: SheetId, at: u32, count: u32) -> Result<(), EditError>;
    pub fn insert_cols(&mut self, sheet: SheetId, at: u32, count: u32) -> Result<(), EditError>;
    pub fn delete_cols(&mut self, sheet: SheetId, at: u32, count: u32) -> Result<(), EditError>;
    pub fn insert_cells(&mut self, range: RangeRef, shift: Shift /* Down | Right */) -> Result<(), EditError>;
    pub fn delete_cells(&mut self, range: RangeRef, shift: Shift /* Up | Left */) -> Result<(), EditError>;
    pub fn move_range(&mut self, from: RangeRef, to: CellRef) -> Result<(), EditError>;   // cut/paste semantics
    pub fn move_rows(&mut self, sheet: SheetId, from: u32, count: u32, to: u32) -> Result<(), EditError>;
    pub fn sort(&mut self, range: RangeRef, keys: &[SortKey], opts: SortOptions) -> Result<(), EditError>;
}

pub struct StructuralChange { pub op: StructuralOp, pub map: RefMap }
impl RefMap {
    pub fn map_cell(&self, c: CellRef) -> Option<CellRef>;      // None = deleted (#REF!)
    pub fn map_range(&self, r: RangeRef) -> RangeMapResult;      // Moved | Resized | Deleted
    pub fn rewrite_formula(&self, formula: &str, host: CellRef) -> String; // for host-held formulas / other workbooks
}
```

Why this is better: GRID returns a `RewriteFormula` string-to-string closure that the caller must apply with the correct sheet and workbook names. RustSheet returns a **data** `RefMap` that also maps host-side anchors such as comments, bookmarks, external IDs and agent memory, and it is serializable for collaboration. IronCalc's `insert_rows` returns `()`. Edit guards are a single typed error family:

```rust
#[non_exhaustive]
pub enum EditError {
    Ref(RefError), Parse(ParseError),
    Blocked { reason: BlockedReason, at: RangeRef },   // SpillRange, MergedCell, TableHeader, ArrayFormulaPart,
                                                        // PivotOutput, DataTableResult, SheetEdge, Protected
    SheetNotFound(String), NameExists(String), InvalidName(String), Cancelled,
}
```

### 4.8 Defined names and tables

```rust
pub enum Scope { Workbook, Sheet(SheetId) }
impl Txn<'_> {
    pub fn define_name(&mut self, name: &str, scope: Scope, formula: &str, opts: NameOptions) -> Result<(), EditError>;
    pub fn rename_name(&mut self, old: &str, scope: Scope, new: &str) -> Result<(), EditError>; // rewrites uses
    pub fn delete_name(&mut self, name: &str, scope: Scope) -> Result<(), EditError>;
    pub fn define_table(&mut self, name: &str, range: RangeRef, opts: TableOptions) -> Result<(), EditError>;
    pub fn table_append_rows(&mut self, table: &str, rows: &[&[Input]]) -> Result<RangeRef, EditError>;
    pub fn resize_table(&mut self, table: &str, range: RangeRef) -> Result<(), EditError>;
}
impl Workbook {
    pub fn names(&self) -> impl Iterator<Item = NameInfo> + '_;   // {name, scope, formula, value, is_lambda, hidden}
    pub fn tables(&self) -> impl Iterator<Item = TableInfo> + '_; // {name, range, columns, header/total rows}
}
```

LAMBDA-valued names can be called as user functions (`=MYFN(1,2)`). Host-registered custom functions use `register_function(name, sig, Arc<dyn Fn(&[ArgValue]) -> Value + Send + Sync>)`, in Rust and in every binding.

### 4.9 Dependency introspection

```rust
impl Workbook /* and Snapshot */ {
    pub fn precedents(&self, r: impl IntoRef, depth: Depth) -> Result<Vec<Ref>, RefError>;  // Direct | Transitive | Max(n)
    pub fn dependents(&self, r: impl IntoRef, depth: Depth) -> Result<Vec<Ref>, RefError>;
    pub fn explain(&self, cell: CellRef) -> Result<Explain, CalcError>;   // evaluation tree: each sub-expression + value
    pub fn formula_refs(&self, formula: &str, ctx: EvalContext) -> Result<Vec<Ref>, ParseError>; // static
    pub fn cycles(&self) -> Vec<Vec<CellRef>>;
    pub fn recalc_plan(&self, targets: &[Ref]) -> RecalcPlan;   // what would be computed, in what order
}
```

The results are ranges, not exploded cells, so `SUM(A:A)` reports one range, as in Excel JS `getDirectPrecedents`. `explain` builds on `XlError.origin` to answer "why is this `#N/A`?". GRID does not expose graph queries publicly (only vertex ids inside errors and volatiles), and IronCalc has none.

### 4.10 Events and subscriptions

Pull is primary, because every commit returns a `ChangeSet`. Push is a thin layer on top:

```rust
impl Workbook {
    pub fn subscribe(&mut self, filter: EventFilter, sink: impl EventSink) -> SubscriptionId;
    pub fn unsubscribe(&mut self, id: SubscriptionId);
}
pub enum Event<'a> {
    Committed(&'a ChangeSet),                 // after every commit/undo/redo/recalc
    WatchedChanged { watch: WatchId, cells: &'a [ValueChange] },
    SheetAdded(SheetMeta), SheetRemoved(SheetId), SheetRenamed { id: SheetId, old: Arc<str> },
    Diagnostic(&'a Diagnostic),
}
pub trait EventSink: Send { fn on_event(&mut self, e: &Event<'_>); }   // closures impl it; channels via adapter
```

Fine-grained per-cell recalc tracing, comparable to GRID's `recalcEvent` and `metrics`, goes through the `tracing` crate (spans per recalc pass, per family, per function, behind a feature flag). It is not exposed through the event API, so there is no runtime cost when unused.

### 4.11 Error model summary

| Layer | Type | Examples |
|---|---|---|
| Excel semantics | `Value::Error(XlError{kind, detail, origin})` | `#DIV/0!` with origin `Sheet1!C7`, `#NAME?` detail `"FOO"` |
| API misuse / refusal | `Result<_, RustSheetError>` with `RefError`, `ParseError{span, expected}`, `EditError::Blocked{reason, at}`, `CalcError::{Cancelled, DeadlineExceeded, ResourceLimit}`, `IoError` | `wb.value("Shet1!A1")` gives `RefError::UnknownSheet("Shet1")` |
| Non-fatal | `Diagnostic { severity: Info\|Warning\|Error, code: &'static str, message, cells: Vec<Ref> }` | unsupported function on load, circular reference created, lossy xlsx feature |

Every error enum is `#[non_exhaustive]`, implements `std::error::Error`, and uses `thiserror`. This replaces IronCalc's pervasive `Result<_, String>`, where callers cannot match on the error kind.

### 4.12 Thread-safety
- `Workbook: Send`. Mutation needs `&mut`, so writers are serialized by the borrow checker; in bindings this becomes a `Mutex`.
- `Snapshot`, `Scenario` and `Calculator` are `Send + Sync + Clone` (`Arc` inside). Many readers or scenario workers run in parallel while the writer keeps editing, giving MVCC-like behavior.
- Internal parallel recalc uses rayon over independent topological levels and formula families, controlled by `Threads`.
- No `'a` lifetimes on public types. IronCalc's `Model<'a>` borrows `&'a Locale`/`&'a Language`; the fork should switch to `&'static` (the data is static tables) or `Arc`, which removes the `leak_str` hack in the Python binding.

### 4.13 XLSX import and export

```rust
impl Workbook {
    pub fn open(path: impl AsRef<Path>, opts: LoadOptions) -> Result<(Workbook, Vec<Diagnostic>), IoError>;
    pub fn from_xlsx_bytes(bytes: &[u8], opts: LoadOptions) -> Result<(Workbook, Vec<Diagnostic>), IoError>;
    pub fn save_xlsx(&self, w: impl Write + Seek, opts: SaveOptions) -> Result<(), IoError>;
    pub fn to_bytes(&self) -> Vec<u8>;                  // native fast snapshot (IronCalc's .icalc bitcode)
    pub fn from_bytes(b: &[u8]) -> Result<Workbook, IoError>;
}
pub struct LoadOptions {
    pub recalc: OnLoad /* TrustCachedValues | RecalcVolatile | RecalcAll */,
    pub calc_mode: Option<CalcMode>, pub sheets: SheetFilter, pub values_only: bool,
    pub preserve_unknown_parts: bool,   // round-trip charts/drawings/VBA we don't model
}
```

Start with IronCalc's `xlsx` crate (`load_from_xlsx`, `save_xlsx_to_writer`). Evaluate **calamine** for a fast read path (cached values with lazy formula parsing), and pass through unknown OOXML parts for round-trip fidelity. Use `TrustCachedValues` with `CalcMode::OnDemand` to open huge files instantly.

### 4.14 Python (PyO3) mapping

```python
import rustsheet as rs
wb = rs.open("model.xlsx", calc_mode="on_demand")
wb["Inputs!B3"]                     # -> 0.07  (value)
wb.cell("Summary!B5").formula       # -> "=B3*12"
with wb.edit(label="scenario") as tx:   # atomic; exception => rollback
    tx["Inputs!B3"] = 0.08
    tx.insert_rows("Data", at=10, count=5)
cs = tx.changes                     # ChangeSet: cs.values -> [(ref, before, after)]
wb.eval("=SUM(Data!A:A)", bindings={"rate": 0.05})
calc = wb.snapshot().calculator(inputs=["Inputs!B3"], outputs=["Summary!B5"])
calc.call_batch(numpy_array)        # releases the GIL (py.allow_threads), parallel
wb.values("Data!A1:D1000")          # -> list[list] or .to_numpy() / .to_arrow() zero-copy for numeric blocks
```

- Excel errors map to a Python `rs.XlError` *value* object (not raised), with `.kind`, `.detail` and `.origin`. API failures raise typed exceptions (`rs.RefError`, `rs.EditBlocked`, and so on).
- `Workbook` is a `#[pyclass]` wrapping `Mutex<Workbook>`; `Snapshot` and `Calculator` are freely shareable.
- Release the GIL around recalc, load and save.

### 4.15 Java mapping (later)

Options, in order of recommendation:
1. **UniFFI** (Mozilla) over a dedicated `rustsheet-ffi` facade crate made of records, enums and objects (`Arc<Workbook>` with an internal mutex). It generates Kotlin (usable from Java), Swift and Python. A community generator `uniffi-bindgen-java` (IronCore Labs) emits plain Java. This is the lowest maintenance path, and it also covers Swift.
2. **Panama FFM** (`java.lang.foreign`, final in JDK 22, JEP 454) over a C ABI produced with `cbindgen` and consumed through `jextract`. This is the fastest path for bulk transfers, since `values()` blocks can be copied into a `MemorySegment` without per-cell JNI calls. It needs JDK 22 or later.
3. **JNI** via the `jni` crate as a fallback for JDK 8 to 21 users. It has the most boilerplate.

The FFI facade is shared by every binding and should only use: ids as `u32`/`u64`, strings, `f64`, enums, flat `Vec`s, and opaque handles. That constraint is why §4.2 avoids generics, lifetimes and trait objects in the public types. `IntoRef` is a Rust-only convenience; bindings pass A1 strings or `(sheet, row, col)` tuples.

### 4.16 What is better than GRID and IronCalc

| Area | GRID | IronCalc | RustSheet |
|---|---|---|---|
| Atomicity | none (snapshot to restore) | none | `edit()` transactions with rollback |
| What changed | not reported | not reported | `ChangeSet` with before/after values, spills, structure |
| Undo | no | UserModel only | built in, derived from ChangeSet inverses |
| Recalc | incremental graph | full re-evaluation | incremental graph, families, parallel, cancellable |
| Lazy / compute-on-read | deferred data tables only | no | `OnDemand` mode, `watch`, `Snapshot::compute` |
| What-if | mutate then restore (best effort) | no | side-effect-free `Scenario`, `Calculator::call_batch` |
| Ad-hoc formula | `runFormula` (opaque cache) | none | `eval` with bindings, `compile` handle |
| Graph introspection | vertex ids, not queryable | none | `precedents`, `dependents`, `explain`, `cycles`, `recalc_plan` |
| Structural edit result | string-rewrite closure | `()` | data `RefMap` (maps cells, ranges, formulas; serializable) |
| Error values | detail + origin (good) | flattened to localized strings | detail + origin kept, typed `ErrorKind` |
| API errors | exception classes | `String` | typed `#[non_exhaustive]` enums |
| Concurrency | single-threaded JS | `&mut`, lifetime-bound | `Send` writer, `Send + Sync` snapshots |
| Determinism | `env` map | test-only mock time | injectable clock, RNG seed, tz |
| Identity | names/indexes | positional sheet index, 1-based `i32` | stable `SheetId`, 0-based `u32` |
| License | evaluation/commercial | MIT/Apache-2.0 | MIT/Apache-2.0 (inherited) |

Ideas deliberately kept from GRID, reworked in Rust form: error `detail` and `origin`; a typed edit-guard family; compact arrays with default fill; formula-inherited number formats (as `CellInfo.effective_format` instead of boxed values); a deferred-data-table calc mode; diagnostics with severity and cell lists; multi-workbook external references (as a later phase via `BookId`); and tracing-based recalc observability.

Ideas deliberately not copied: the string-keyed event emitter, JSF-style one-letter cell fields (`v`, `f`, `z`, `M`), a model-wide formula-text cache, and closure-based formula rewriting.

### 4.17 Suggested phasing on the IronCalc fork
1. **API facade** over the current IronCalc `Model`: typed errors, `SheetId`, 0-based refs, `Value::Error`, the `edit()`/`ChangeSet` shape (computed by diffing before and after a full `evaluate()`), `eval()` via a hidden scratch cell or direct AST evaluation, and the Python binding. This ships something useful quickly.
2. **Dependency graph and incremental recalc**, so that `ChangeSet` comes from dirty propagation instead of diffing. Add precedents and dependents.
3. **Snapshots (chunked copy-on-write storage)**, `OnDemand` mode, scenarios and `Calculator`.
4. **Formula families**, parallel recalc and cancellation; benchmark against Formualizer and LibreOffice.
5. UniFFI/Panama Java bindings, multi-workbook external links, round-trip preservation of unknown parts.

---

### Sources
- GRID: https://docs.grid.is/ , https://docs.grid.is/spreadsheet-engine/ , https://docs.grid.is/spreadsheet-engine/tips/ , https://docs.grid.is/spreadsheet-engine/api/classes/model/ , …/workbook/ , …/worksheet/ , …/cell/ , …/range/ , …/reference/ , …/table/ , …/tablemanager/ , …/matrix/ , …/lambda/ , …/formulaerror/ , …/modelerror/ , …/editblockederror/ , …/valuesnapshot/ , …/vertexid/ , …/cellvertexid/ , …/namevertexid/ , …/vertexidset/ , https://docs.grid.is/spreadsheet-engine/api/interfaces/writeoptions/ , …/interfaces/evaluateexpressionoptions/ , …/interfaces/evaluationcontext/ , …/type-aliases/cellvalue/ , …/type-aliases/formulavalue/ , …/type-aliases/modeleventargs/ , …/type-aliases/recalcevent/ , …/type-aliases/rewriteformula/ , https://grid.is/engine , https://grid.is/blog/developers-meet-the-spreadsheet-engine-you-didnt-know-you-needed , https://docs.grid.is/apiary/tutorials/quickstart/
- IronCalc: https://github.com/ironcalc/IronCalc , https://www.ironcalc.com/roadmap.html , https://blog.ironcalc.com/ , https://nlnet.nl/project/IronCalc/ , and local source `base/src/model.rs`, `base/src/evaluation.rs`, `base/src/cell.rs`, `base/src/calc_result.rs`, `base/src/user_model/common.rs`, `bindings/python/src/raw_model.rs`, `xlsx/src/{import,export}/mod.rs`
- Formualizer: https://github.com/psu3d0/formualizer , https://www.formualizer.dev/docs , https://crates.io/crates/formualizer (public API snapshot `public-api/formualizer-workbook.txt` and `formualizer-eval.txt` in the repo)
- HyperFormula: https://github.com/handsontable/hyperformula , https://hyperformula.handsontable.com/api/classes/hyperformula.html , https://hyperformula.handsontable.com/docs/guide/release-notes.html , https://hyperformula.handsontable.com/docs/guide/licensing.html , https://github.com/handsontable/hyperformula/discussions/1268
- Univer: https://github.com/dream-num/univer , https://docs.univer.ai/guides/sheets/getting-started/node , https://docs.univer.ai/guides/sheets/features/core/formula , https://reference.univer.ai/en-US
- LibreOffice: https://docs.libreoffice.org/sc/html/formulacell_8cxx_source.html , https://wiki.openoffice.org/wiki/Calc/Implementation/Formula_cell_and_cells_dependence , https://conference.libreoffice.org/assets/Conference/Rome/Slides/libocon-2017-novideos.pdf
- Apache POI: https://poi.apache.org/components/spreadsheet/eval.html , https://poi.apache.org/apidocs/dev/org/apache/poi/ss/usermodel/FormulaEvaluator.html
- Python engines: https://github.com/dgorissen/pycel , https://github.com/vinci1it2000/formulas , https://github.com/bradbase/xlcalculator
- EtherCalc: https://github.com/audreyt/ethercalc
- Rust IO crates: https://github.com/MathNya/umya-spreadsheet , https://github.com/tafia/calamine , https://github.com/jmcnamara/rust_xlsxwriter
- Excel JS API: https://learn.microsoft.com/en-us/javascript/api/excel/excel.range?view=excel-js-preview , https://learn.microsoft.com/en-us/javascript/api/excel/excel.worksheet?view=excel-js-preview , https://learn.microsoft.com/en-us/office/dev/add-ins/excel/excel-add-ins-ranges-insert
- Bindings: https://mozilla.github.io/uniffi-rs/ , https://github.com/IronCoreLabs/uniffi-bindgen-java , https://openjdk.org/jeps/454 , https://pyo3.rs/
