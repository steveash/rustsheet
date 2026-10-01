# RustSheet: Excel-compatibility test suites, corpora, benchmarks, and Excel-as-oracle

> Research snapshot produced at project inception (2026-10-01) by research agents. Treat as background, not as normative spec; normative docs live in `docs/architecture/` and `docs/process/`. Update via a PR when findings change.

Date: 2026-10-01. Target: exact parity with current Microsoft 365 Excel (en-US, 1900 date system, dynamic arrays). Project license: Apache-2.0.

Method: I shallow-cloned and inspected the repos where I could (IronCalc local clone, Formualizer, HyperFormula, LibreOffice `sc/qa/unit/data/functions`, Apache POI `test-data/spreadsheet`, witanlabs/xlsx-corpus-bench, Gnumeric, xlcalculator, formulas, pycel, formulajs). The rest is from web docs. Numbers marked "measured" come from scripts I ran on the clones. Clones are in `scratchpad/ext/`.

---

## TL;DR

1. **Key insight confirmed: every xlsx saved by Excel is a free oracle dataset.** I measured Excel-written `<v>` caches. Excel writes the shortest round-trip decimal, up to 17 significant digits (for example `0.30000000000000004`, `-9.9999999999999998E-13`), so the cached value is the exact binary64. This held for Windows, Mac and Excel Online writers. IronCalc's whole function test strategy already works this way: load an Excel-authored xlsx, recalculate, and diff against the cached values. **Filter by provenance** (`docProps/app.xml` `<Application>`, `calcPr`), not by file extension.
2. **Best value-to-effort for correctness:**
   - (a) IronCalc `xlsx/tests/calc_tests`: 207 Excel-authored function workbooks with about 71k formula cells. MIT/Apache, so we can vendor them now.
   - (b) Apache POI `test-data/spreadsheet`: 832 files, about 31k formula cells in the xlsx files alone, plus 417 xls files, mostly Excel-authored. Apache-2.0, so we can vendor them.
   - (c) SpreadsheetBench, about 5.4k real forum workbooks (CC BY-SA, fetch at test time), plus FUSE (10.7k OOXML of 249k files, CC-BY 4.0, 9.4 GB, fetch at test time).
   - (d) LibreOffice function fods files: 516 self-checking files with about 5.2k check rows. MPL-2.0, fetch at test time. They need ODF-to-Excel formula translation. Their expected values are mostly, but not entirely, Excel-derived.
3. **The witanlabs/xlsx-corpus-bench repo (Apache-2.0) is a ready blueprint and a public leaderboard.** It covers 15,970 FUSE and SpreadsheetBench workbooks, uses real Excel (Mac, AppleScript, `fullCalcOnLoad` injection) as truth, and applies a 1e-9 relative tolerance comparator. LibreOffice scores 96.7–98.9% of cells and witan scores 99.7–99.8%. It does **not** publish Excel's values, only per-file mismatch counts, so we must regenerate truth ourselves. The recipe is about 300 lines of Python we can port.
4. **Oracle: use Microsoft Graph only if the owner has a *work/school* (Entra ID) tenant.** The Graph Excel API does not support OneDrive consumer or personal Microsoft accounts. Delegated `Files.ReadWrite` is the documented path. App-only (client credentials) is documented as "Not supported" for `calculate` and `usedRange`, but users report it works. Plan for delegated device-code login plus a refresh-token cache, and test app-only as an optional upgrade.
5. **The most faithful and highest-throughput oracle is a Windows box with desktop Excel over COM** (or Mac Excel over AppleScript, as witan does). Microsoft does not support it for unattended use, but it is the only way to get desktop-only functions (FILTERXML, WEBSERVICE, CELL/INFO variants) and exact parity with "desktop M365".
6. **Architecture:** a content-addressed oracle cache (`sha256(xlsx)` → JSON of every formula cell's Excel value, numbers stored as exact round-trip strings). It has pluggable backends: `cached-v` (free), `graph`, `excel-com`, `excel-mac`, and `libreoffice` (approximate, labeled). Golden JSON files are committed to the repo so agents can work offline. The oracle only runs when the cache misses.

---

# PART A — Test suites, corpora, benchmarks

## A.0 The "cached values are an oracle" approach, and its pitfalls

Every `<c>` with an `<f>` in a SpreadsheetML worksheet carries the last computed result in `<v>`, with a `t=` type: `n` default, `str` formula string, `b` boolean, `e` error, `s` shared string (rare for formulas). Dynamic-array anchors carry `<f t="array" ref="A1:C5">` plus `cm="1"` cell metadata. Spilled cells carry only `<v>`, so treat every cell inside a dynamic `ref` as an expected output. Recalculate in RustSheet and diff.

**Measured precision (IronCalc corpus, digit counts of cached numeric formula results):**

| Writer | 15 sig digits | 16 | 17 |
|---|---|---|---|
| Microsoft Excel (Win) | 15 | 68 | 929 |
| Microsoft Macintosh Excel | 21 | 27 | 416 |
| Microsoft Excel Online | 1 | 28 | 364 |

Examples: `0.30000000000000004`, `7.9308261160397242E-2`, `9.9999999999999999E+306`. Excel serializes the shortest representation that round-trips (G17-style), **not** the 15-digit display value. So `parse::<f64>(v)` gives the exact double Excel computed. Compare bit-exact first, then fall back to a tolerance and report both rates. Contrast this with Excel's *display* and the `=` operator, which work at 15 significant digits. The calipers project likewise accepts "equal at 15 sig digits" as a secondary match. Note that Excel's 15-digit `=` and its "close to zero" snapping in `+` and `-` are semantics we must implement, not tolerance excuses.

**Pitfalls and mitigations:**

| Pitfall | Detection / mitigation |
|---|---|
| File not saved by Excel. openpyxl writes no `<v>`; **XlsxWriter writes `<v>0</v>` for every formula unless the caller supplies a value**; LibreOffice, POI, EPPlus, Google Sheets and Numbers write their *own* engine's values | Accept only if `docProps/app.xml` `<Application>` ∈ {`Microsoft Excel`, `Microsoft Macintosh Excel`, `Microsoft Excel Online`, `Excel Android`/iOS?}. Measured in POI: 265 Win / 31 Mac / 7 Online, and 6 POI / 8 LibreOffice / 1 ODF Converter files that must be excluded. Treat a missing `app.xml` as untrusted. |
| Stale caches | `calcPr calcMode="manual"` (and `calcOnSave="0"`) mean caches may be stale, so exclude or oracle-recalculate. `fullCalcOnLoad="1"` means the producer told Excel *not* to trust caches, so the file was usually machine-written. |
| Old calc engine | `calcPr calcId` records the engine that last calculated, for example 125725 = 2007, 145621 = 2010, 152511 = 2013, 162913 = 2016, 19xxxx = current M365. Files from old engines can differ from current M365. Function semantics have changed over time (for example, statistical function accuracy improvements in 2010). Bucket results by calcId. `AppVersion` is 16.0300 for every Excel since 2016, so it is useless as a version signal. |
| Volatile / environment functions | Exclude cells whose *own* formula contains NOW, TODAY, RAND, RANDBETWEEN or RANDARRAY (witan's rule), and cells depending on CELL("filename"), INFO, WEBSERVICE, RTD, STOCKHISTORY, IMAGE, linked data types, or `=PY(`. OFFSET and INDIRECT are volatile but deterministic, so keep them. IronCalc mocks NOW from a `METADATA` sheet. Copy that convention for our own generated tests. |
| Precision-as-displayed | `calcPr fullPrecision="0"` changes stored values. Implement it or exclude the file. |
| Iterative calc / circular refs | `calcPr iterate="1"` with `iterateCount`/`iterateDelta`. Bucket these files separately. |
| 1904 date system | `workbookPr date1904="1"`. Supported by Excel, but it is outside our target. Implement it or exclude the file. |
| External links | `xl/externalLinks/*.xml` hold cached external values. Excel's closed-workbook semantics (for example `SUMIF` over a closed ref → `#VALUE!`) are the truth when it recalculates offline. Keep these files and model the semantics, or exclude them. |
| Data tables `{=TABLE()}`, Goal Seek results, pivot `GETPIVOTDATA` | Bucket these separately. |
| Locale | Formulas in xlsx are stored locale-invariant (en-US names, `,` separators), but **results** depend on the author's locale: TEXT() format codes, DATEVALUE/VALUE parsing of strings, currency symbols, list separators in TEXTJOIN data, and so on. Corpus files authored in de-DE or fr-FR will "fail" for correct reasons. Heuristics: number formats with locale tags `[$-407]`, the `<Language>` element in `docProps`, or non-English sheet names. |
| `_xlfn.`/`_xlws.` prefixes & `#NAME?` | A file saved by an older Excel containing a newer function caches `#NAME?`. Excel M365 recalc would not. Detect `#NAME?` caches on `_xlfn.` functions and route those cells to the live oracle. |
| calcChain.xml | Only an ordering hint. Ignore it, but its presence is a weak "Excel saved this" signal. |
| Display vs stored | Compare `<v>`, never the formatted text. For TEXT/format-dependent tests, compare the formula's string result (that is a value). Use the oracle's `text` output only if we test number formatting. |
| Shared strings / rich text | `t="s"` constants are inputs. Formula string results are `t="str"` inline. Do not compare shared-string indices (IronCalc has a FIXME on exactly this). Compare resolved text. |
| Errors | `t="e"` with text such as `#N/A`, `#SPILL!`, `#CALC!`, `#BLOCKED!`, `#FIELD!`, `#CONNECT!`, `#BUSY!`, `#UNKNOWN!`, `#PYTHON!`. Compare by error code. |
| Unsupported cells | Excel sometimes leaves cells uncalculated. witan documents about 0.08% of cells and 0.13% of files in SpreadsheetBench where Excel truth is imperfect, which is also why a live Excel oracle is still needed. |

---

## A.1 Inventory

Legend for "Vendor?": **Y** = can commit into an Apache-2.0 repo (keep the NOTICE/attribution). **Fetch** = download at test time, for example `scripts/fetch-corpora.sh` into `~/.cache`, never committed. **Ref** = read for ideas only, do not copy.

### 1. IronCalc `xlsx/tests` (local clone)
- URL: https://github.com/ironcalc/IronCalc (local clone: `scratchpad/IronCalc`)
- License: MIT OR Apache-2.0 → **Y**, with attribution.
- Size/format (measured): `xlsx/tests/calc_tests/` holds 207 xlsx (3.5 MB) in category folders. Counts: STATISTICAL 50, MATH_AND_TRIG 31, FINANCIAL 29, LOOKUP_AND_REFERENCE 20, TEXT 19, DATE_AND_TIME 13, LOGICAL 11, INFORMATION 9, ENGINEERING 7, DATABASE 4, plus 14 top-level files (arithmetic, defined names, spill operator, range operator, simple arrays, escape strings, ...). There are 14 more in `tests/docs/` (function doc examples) and 1 in `calc_test_no_export/` (tables). Templates (invoice, gantt, calendar, ...) are in `tests/templates/`. Total formula cells across these: **≈71,000** (counting `<f>` elements, so shared-formula children are included).
- Provenance (measured): 119 Mac Excel, 72 Win Excel, 26 Excel Online, 1 Excel Android, 2 IronCalc-written (exclude those 2).
- Covers dynamic-array and modern functions: XLOOKUP/XMATCH, FILTER, SORT, TOCOL, WRAPCOLS/ROWS, HSTACK/VSTACK, CHOOSECOLS/ROWS, DROP/TAKE, EXPAND, TRIMRANGE, TEXTSPLIT, TEXTBEFORE/AFTER, REGEX, ARRAYTOTEXT, VALUETOTEXT, spill operator, and more.
- Extraction: cached `<v>`. Harness: `xlsx/src/compare.rs` `test_file()` loads the file twice, evaluates one copy and diffs every cell. Conventions: a `METADATA` sheet with an epsilon in `METADATA!A1` (default relative eps) and a mocked "NOW" timestamp row. `build.rs` generates one `#[test]` per file.
- Effort: **very low.** Needs only our xlsx reader plus the diff harness. This is the first milestone.

### 2. Apache POI `test-data/spreadsheet`
- URL: https://github.com/apache/poi/tree/trunk/test-data/spreadsheet (sparse clone: `scratchpad/ext/poi`)
- License: Apache-2.0 (ASF distributes the test data with the source) → **Y**. Spot-check bug-report files before vendoring them all. The safest option is to vendor only the function-test workbooks.
- Size (measured): 832 files, 68 MB: 417 xls, 352 xlsx, 14 xlsm, 11 xlsb. The xlsx/xlsm subset has 30,765 formula cells in 130 files. 18 files are deliberately broken zips (negative tests).
- Key files: `FormulaEvalTestData.xls` (the master function test sheet with ROW/FORMULA/expected layout, used by POI's `TestFormulasFromSpreadsheet`), `FormulaEvalTestData_Copy.xlsx`, `MatrixFormulaEvalTestData.xls(x)`, and ~25 `*FunctionTestCaseData.xls` files (Index, Indirect, Match, Lookup, Rept, Roman, Quotient, Complex, WeekNum, WeekNum2013, Logical, Boolean, Code, Delta, FactDouble, If, ImReal, Imaginary, Row, Value, TwoOperandNumeric, 57003-FixedFunction, ...).
- Format: mostly BIFF8 .xls. calamine reads xls cached values, or convert once via the Excel oracle to xlsx. Expected values sit in explicit "expected" columns and also in the caches.
- Effort: low for the xlsx files, medium for xls (we need an .xls reader, or convert once with the oracle and vendor the xlsx output).

### 3. LibreOffice `sc/qa/unit/data/functions`
- URL: https://github.com/LibreOffice/core/tree/master/sc/qa/unit/data/functions (sparse clone: `scratchpad/ext/lo`)
- License: MPL-2.0 (file-level copyleft). We *may* include unmodified MPL files in an Apache repo, but they stay MPL. Simplest is **Fetch**.
- Size (measured): 516 `.fods` (flat ODF XML), 117 MB. Counts: statistical 147, mathematical 79, financial 51, addin (Analysis ToolPak functions) 49, spreadsheet 45, text 44, date_time 32, information 20, array 15, database 12, logical 9, lambda 8 (LAMBDA/MAP/REDUCE/SCAN/BYROW/BYCOL/MAKEARRAY/ISOMITTED), plus `fods/` misc including `Functions_Excel_2016.fods`. `dynamic_array/xlsx/` has 2 Excel-Online-saved xlsx. **≈5,240 check rows** (measured by counting `ROUND(A;12)=ROUND(B;12)` checks).
- Layout: Sheet1 holds an overall `AND(...)` pass flag. Sheet2 is a table with column A = formula, B = expected literal, C = check, D = FORMULA(A) text. Formulas are ODF syntax (`of:=AVERAGE([.G1:.G15])`, `;` separators).
- Expected values: literals typed by the LO QA team. Many were cross-checked against Excel 2010–2016 but they are **not guaranteed Excel**. Some files encode LO-specific behavior, and some `.xcl.` variants encode Excel-compatible variants. Use them as *input generators* and re-derive expected values with our Excel oracle.
- Derived work: workspace-sh/table-file-format PR #134 converted 2,874 of these cases into EDN with a spec-grammar reader, and noted 8,734 more that need cell inputs. Reference: https://github.com/workspace-sh/table-file-format/pull/134
- Effort: medium. Write an ODF formula → Excel A1 translator, regenerate expected values via the oracle, and commit our own generated xlsx and golden JSON (we own those).

### 4. Gnumeric test suite
- URL: https://gitlab.gnome.org/GNOME/gnumeric (`samples/excel/*.xls`, `test/t10xx-*.pl`)
- License: GPL-2.0+/GPL-3 → **Fetch / Ref**.
- Content: classic self-checking Excel function workbooks (`statfuns.xls`, `mathfuns.xls`, `finfuns.xls`, `engfuns.xls`, `textfuns.xls`, `logfuns.xls`, `datefuns.xls`, `dbfuns.xls`, `infofuns.xls`, `lookfuns.xls`, `lookfuns2.xls`, `functions.xls`) whose pass cell (for example A4 = "All ok") was computed by Excel. There are also xlsx files for countif, database and ifs-funcs, and `.gnumeric` tests for xlookup, regex*, switch and unique, plus numerical-accuracy suites (yalta2008, burkardt, crlibm, gsl).
- Extraction: the xls caches (Excel-authored) work directly.
- Effort: low–medium (xls reader). Very good statistical, financial and engineering edge cases. Do not copy.

### 5. HyperFormula
- URL: https://github.com/handsontable/hyperformula
- License: GPL-3.0 or commercial → **Ref only**.
- Status (verified): the test suite now lives in the **private** repo `handsontable/hyperformula-tests` (`test/fetch-tests.sh` clones it with a deploy token). Public tags up to about 2.x still contain `test/interpreter/*.spec.ts`: 471 spec files at tag 2.6.0, roughly one per function, as inline `expect(engine.getCellValue(...)).toEqual(...)`. The expected values are HyperFormula's, which are mostly Excel-aligned, but HyperFormula documents deliberate differences.
- Useful public docs: "Compatibility with Microsoft Excel" and "List of differences" (`docs/guide/compatibility-with-microsoft-excel.md`, `list-of-differences.md`). These are a good checklist of Excel quirks: string comparison, accent sensitivity, empty-cell handling, and more.
- Perf: `test/performance/sheets/*.ts` holds synthetic benchmark sheet generators.
- Effort: we cannot vendor anything. Re-implement case ideas and verify them via the oracle.

### 6. Formualizer
- URL: https://github.com/PSU3D0/formualizer
- License: MIT OR Apache-2.0 → **Y**.
- Content (verified): function tests are mostly inline Rust unit tests in `formualizer-eval`, with expected values written by the authors (not Excel-verified). There is a small `tests/corpus/` of JSON cases (cycles, dynamic arrays, named ranges) and two xlsx fixtures. The **benchmark infrastructure** is the valuable part: `benchmarks/scenarios.yaml` (families include incremental_locality, chain_topology, lookup_dimension_join, aggregate_analytics, structural_edit, real_world_anchor and nightly_stress, with tiers pr_smoke, comparative and nightly_heavy), a synthetic xlsx corpus generator (`formualizer-bench-core generate-corpus`), deterministic "real-world-shaped" anchors (finance and ops models) with expected outputs, a HyperFormula node adapter, and large-workbook envelope probes (up to about 700k rows).
- Also: `docs/cache-only-xlsx.md` is a careful write-up of rewriting `<v>` caches. Its eligibility rules (no tables, no spills, ...) are a good pitfall list.
- Effort: low for porting the perf scenarios. Its correctness tests are only moderately useful.

### 7. xlcalculator / pycel / formulas (Python)
- xlcalculator (MIT, **Y**): https://github.com/bradbase/xlcalculator. `tests/resources/*.xlsx` holds 66 Excel-authored function workbooks (ABS, ACOS, ..., CONCAT, ...) and `tests/xlfunctions_vs_excel/` compares against Excel caches. These are small but free.
- pycel (GPL-3, **Ref**): https://github.com/dgorissen/pycel. 14 fixture xlsx (basic, circular, date-time, information, logical, lookup, npv, pv, ...).
- formulas (EUPL-1.1+, **Fetch/Ref**): https://github.com/vinci1it2000/formulas. 5 test xlsx, including a large multi-function `excel.xlsx` and `circular.xlsx`.
- Effort: low. Run them through the provenance filter and the diff harness.

### 8. formulajs
- URL: https://github.com/formulajs/formulajs. License: MIT → **Y** (cases).
- Content: JS unit tests per function with hand-written expected values (no xlsx). They have many known deviations from Excel. Use them only as **case generators** with oracle-verified expected values.
- Effort: medium (JS → our case format).

### 9. witanlabs/xlsx-corpus-bench (highly relevant)
- URL: https://github.com/witanlabs/xlsx-corpus-bench. License: Apache-2.0 → **Y** for the harness.
- What it is: "Benchmarking xlsx libraries against Microsoft Excel itself" across FUSE (10,544 workbooks, 5.67M formula cells) and SpreadsheetBench (5,426 workbooks, 1.16M formula cells), with **Excel as truth**. The truth harness (`harness/excel_truth.py`) copies each file, injects `fullCalcOnLoad="1"` into `<calcPr>`, opens and Save-As's it in Mac Excel via AppleScript in chunks of 10 with timeouts, kills Excel on hangs and records skips. The saved file's caches are the truth. The comparator (`harness/cached_values.py`) uses 1e-9 relative tolerance for numbers, exact strings and error codes, excludes the 5 volatile functions, and counts missing cells as mismatches.
- Results (current README): on FUSE, witan matches 99.7% of cells, LibreOffice 98.9%, ClosedXML 69.0% and EPPlus 52.2%. On SpreadsheetBench, witan matches 99.8% and LibreOffice 96.7%.
- Published data: per-file JSONL (sha256, formula_cells, mismatches, ...) and sha256 manifests. It does **not** publish Excel's computed values, so we regenerate them.
- Effort: low. Port the manifest, fetch and comparator, then add a `rustsheet` runner. This gives us a public, comparable scoreboard on day one.

### 10. SpreadsheetBench (LLM benchmark; corpus value only)
- URL: https://github.com/RUCKBReasoning/SpreadsheetBench. License: **CC BY-SA 4.0** → **Fetch**. ShareAlike makes vendoring awkward.
- 912 instructions, 2,729 test-case workbooks (5,455 unique by sha256, about 100 MB) from Excel forum questions. They are modern, formula-heavy, and use lots of lookup, text, date and dynamic array functions. The "answer" files were produced by humans or LLM pipelines and recalculated with LibreOffice or win32com, so **their caches are not all Excel-trustworthy**. Use the oracle (or witan's provenance approach). The LLM task itself is irrelevant to us.
- Related LLM benchmarks (SheetCopilot, SpreadsheetBench 2, Finch, MBABench/WorkstreamBench, FLARE, AIDABench) are mostly about agents. They add few new workbooks and often have restrictive licenses. Their relevance is low, apart from Finch-style real finance models as perf and correctness anchors if the license permits.

### 11. FUSE corpus
- URL: https://zenodo.org/records/581678 (single `fuse.zip`, 9.4 GB). License: **CC-BY-4.0** → **Fetch**. It is too big to vendor anyway. We may vendor small curated subsets with attribution.
- 249,376 unique spreadsheets from Common Crawl. Only about 10.7k are OOXML (the rest are .xls). The data is web-sourced and old, from every producer imaginable, so apply the provenance filter. Witan's `fuse_extract.py` does the OOXML detection.
- Effort: medium (storage, filtering). The .xls majority (~238k) is usable once we can read xls caches. BIFF caches are full doubles.

### 12. Enron corpus (Hermans & Murphy-Hill 2015)
- URL: https://figshare.com/articles/dataset/Enron_s_Spreadsheets_and_Related_Emails_A_Dataset_and_Analysis/1222882. License: CC BY 4.0 (per figshare/search) → **Fetch**. It also contains PII from emails, so never vendor it.
- 15,770 spreadsheets, of which 9,120 have formulas: **20.3M formula cells**, 913k unique formulas. Nearly all are Excel 97–2003 `.xls` saved by Excel 97–2002. Caches therefore come from a *very old* calc engine (differences in statistical functions, rounding, and so on). Use them for perf, parser robustness and coverage, and as oracle inputs. Do not treat their caches as M365 truth without bucketing. 76% of sheets use only 15 functions, so this is great for "common path" correctness.
- VEnron (https://dl.acm.org/doi/10.1145/2889160.2889238, project page castle.cse.ust.hk/venron): 7,294 versioned Enron spreadsheets in 360 evolution groups. Useful for **incremental recalc** benchmarks (successive versions). Fetch.
- EUSES (about 4,037–4,498 web .xls files from ~2005): license unclear or research-only, so Fetch/Ref. Low priority, because Enron and FUSE supersede it.

### 13. SheetJS test_files
- Apache-2.0, but **the GitHub repo is disabled** (HTTP 403, "access disabled by GitHub staff due to excessive resource use") and git.sheetjs.com returns 404 for it. The files are mostly parser and format edge cases with few formulas. Low value now. Skip unless a mirror appears.

### 14. calipers (fundamental-research-labs)
- URL: https://github.com/fundamental-research-labs/calipers. License: MIT → **Y**.
- A CLI for "Excel-golden generation and Excel-vs-engine verification": Windows Excel COM load+save, or Office.js scripts, produce `golden.xlsx` plus `.meta.json` with the Excel build. It has 247 cases (83 roundtrip, 144 Office.js, ...). It warns *not* to generate goldens on Mac, because Mac Excel serializes OOXML differently. That matters for round-trip, less so for values. Its numeric comparison accepts bit-exact **or** 15-significant-digit agreement (PR #46).
- Effort: low. It is a second reference implementation of the oracle-golden workflow.

### 15. OASIS OpenFormula / ODFF
- Spec: https://docs.oasis-open.org/office/v1.2/OpenDocument-v1.2-part2.html. The OpenFormula spec has inline test cases (expression → expected), designed to be extracted. The license is OASIS copyright (copying permitted for implementation), so **Ref/Fetch**.
- Semantics are ODF, not Excel. They differ on, for example, empty-string coercion, some function domains, and Excel's 1900 leap-year bug. Use them as case generators only, with oracle-derived expected values. Low–medium value.

### 16. Microsoft support function pages
- Example: https://support.microsoft.com/en-us/office/product-function-8e6b5b24-90ee-4650-aeec-80982a0512ce. Each of roughly 500 pages has "Copy the example data in the following table, and paste it in cell A1" plus Formula / Description / Result rows.
- License: Microsoft Terms of Use, not open → **do not vendor the text**. Scrape locally into (data grid, formulas). Facts like `=PRODUCT(A2:A4)` are short and functional. Then **re-derive results via the Excel oracle**. The pages contain known typos and outdated results, so treat the "Result" column only as a cross-check signal.
- Value: very broad function coverage (every function, including the obscure ones) at low cost. Effort: medium (HTML table parsing and quirks).

### 17. Excel's own templates
- create.microsoft.com templates are Microsoft-copyrighted with no redistribution grant → **do not vendor**. Fetch locally as perf and realism inputs only. IronCalc already ships its own templates (MIT/Apache) in `xlsx/tests/templates/`.

### 18. Calc-engine performance benchmarks
- No canonical public "large financial model" benchmark exists. Practical sources:
  - Formualizer's scenario catalog and generator (Y), with chain, fan-out, SUMIFS-report, lookup-join, whole-column and 100k-cell headline scenarios.
  - HyperFormula's `test/performance/sheets` generators (GPL, ideas only).
  - Real workbooks: the largest Enron and FUSE files by formula count (pick the top 100 by `<f>` count and dependency depth).
  - VEnron version pairs for incremental recalc.
  - Synthetic Excel-classic stress tests: deep chains of 1M cells, 10k×100 SUMIFS grids, volatile-heavy OFFSET/INDIRECT, full-column references, LET/LAMBDA recursion, large spills (SEQUENCE(1e6)), and XLOOKUP on 1M rows (sorted and unsorted).
  - Also time the Excel oracle itself (COM `Application.CalculateFullRebuild` timed by `QueryPerformanceCounter`, Charles Williams' FastExcel methodology) to get "Excel desktop seconds" reference numbers on the same hardware.

---

## A.2 Ranking

### (1) Correctness hill-climbing (value ÷ effort)
1. **IronCalc calc_tests** (Y, ~71k cells, Excel-authored, modern functions). Wire it up on day 1.
2. **Apache POI function test data** (Y; xlsx now, xls after the reader or a one-time oracle conversion).
3. **xlcalculator resources** (Y, tiny, free) and **calipers cases** (Y).
4. **witan harness + SpreadsheetBench** (Fetch, 5.4k real workbooks, ~1.2M formula cells) with **oracle-regenerated truth**. This is the "real world" KPI, comparable to the published LibreOffice and witan scores.
5. **Gnumeric samples/excel** (Fetch, Excel-authored self-checking xls; strong numeric edge cases).
6. **Microsoft support examples**, scraped and then oracle-verified (breadth over all ~500 functions).
7. **LibreOffice fods**, translated and then oracle-verified (~5.2k targeted edge cases).
8. **FUSE OOXML subset** (10.5k; web-scale diversity), later the FUSE and Enron xls bulk.
9. formulajs / HyperFormula / OpenFormula case ideas, oracle-verified.

A meta-strategy is a **differential fuzzer**: generate random formulas from a grammar plus function signatures, batch tens of thousands per workbook, and send them to the oracle once. This becomes the largest source of novel edge cases after the corpora are exhausted.

### (2) Performance benchmarking
1. Formualizer scenario catalog and generator (Y): ready-made families, tiers, and a HyperFormula comparison adapter.
2. Largest real workbooks from Enron and FUSE (Fetch). Pick by formula count and dependency depth, and record load+full-calc+incremental-edit timings.
3. VEnron version pairs (incremental recalc).
4. Hand-written stress set (dynamic arrays, lookups, volatile functions, full-column references).
5. Excel desktop timings on the oracle box as the reference line.

---

# PART B — Excel as an oracle, headless and automated

## B.1 Microsoft Graph Excel REST API (Excel for the web's calc service)

**Hard constraints (from the docs):**
- "Support for workbooks stored in OneDrive Consumer platform is still not available. At this time, only the files stored in business platform are supported by Excel REST APIs." Only `.xlsx` (OOXML) is supported; `.xls` is not. Source: https://learn.microsoft.com/en-us/graph/api/resources/excel
- Permissions for `workbookApplication: calculate` and `worksheet: usedRange` are documented as: Delegated (work or school) = `Files.ReadWrite`; Delegated (personal Microsoft account) = **Not supported**; Application = **Not supported**. Sources: https://learn.microsoft.com/en-us/graph/api/workbookapplication-calculate , https://learn.microsoft.com/en-us/graph/api/worksheet-usedrange
- Microsoft Q&A reports that app-only tokens *do* work against `/users/{id}/drive/items/{id}/workbook/...`, but this is unofficial and could break. Source: https://learn.microsoft.com/en-us/answers/questions/2143028/ms-graph-workbookapplication-calculate-api-with-ap
- So **if the owner's "M365 account" is Personal or Family, Graph will not work.** They need a work/school tenant. Options:
  - Microsoft 365 Business Basic, the cheapest plan with OneDrive for Business and Excel for the web.
  - The Microsoft 365 Developer Program E5 sandbox, which is now limited to Visual Studio Pro/Enterprise subscribers, some partner programs, and Premier/Unified support customers (https://learn.microsoft.com/en-us/office/developer-program/join-with-visual-studio).

**Sessions, limits, throttling:**
- Create a session with `POST .../workbook/createSession {"persistChanges": false}` for a non-persistent scratch copy, ideal for an oracle, or `true` to save. Pass `workbook-session-id` on later calls. Persistent sessions expire after about 5 minutes idle and non-persistent ones after about 7 minutes. Large workbooks: send `Prefer: respond-async` and receive 202 with a Location to poll (roughly every 30 s, at most every 4 min). Source: https://learn.microsoft.com/en-us/graph/workbook-best-practice
- Throttling: Excel APIs are limited to 5,000 requests per 10 s per app across all tenants and 1,500 per 10 s per app per tenant (https://learn.microsoft.com/en-us/graph/throttling). Docs: "We don't recommend increasing concurrency... for each workbook, only send the next request after receiving a successful response." Respect `Retry-After` on 429/503. Error taxonomy: https://learn.microsoft.com/en-us/graph/workbook-error-handling (`unsupportedWorkbook`, `requestAborted` for long calculation, `invalidSessionReCreatable`, ...).
- Payload: Excel on the web caps each request and response at **5 MB**, and a range read at **5,000,000 cells** (https://learn.microsoft.com/en-us/office/dev/add-ins/concepts/resource-limits-and-performance-optimization). Page `usedRange` reads in row blocks.
- File size: SharePoint and Excel for the web will not open workbooks over **100 MB** (https://learn.microsoft.com/en-us/office365/servicedescriptions/office-online-service-description/excel-online). Graph simple upload (`PUT /content`) handles files up to about 250 MB. Use an upload session above that, though the workbook API will refuse such files anyway.
- JSON batching (`POST /$batch`) allows up to 20 sub-requests per batch, with `dependsOn` for ordering. It is useful for reading many sheet ranges in one round trip. Keep a single workbook's requests sequential.

**Oracle recipe (per workbook):**
1. `PUT /me/drive/root:/rustsheet-oracle/{sha256}.xlsx:/content` (or `/drives/{driveId}/items/{folderId}:/{sha}.xlsx:/content`).
2. `POST .../workbook/createSession {"persistChanges": true|false}`, with `Prefer: respond-async` for big files.
3. `POST .../workbook/application/calculate {"calculationType":"FullRebuild"}`. Values are `Recalculate | Full | FullRebuild`; FullRebuild also rebuilds the dependency tree. Also consider pre-injecting `fullCalcOnLoad="1"` (witan's trick) so the open itself does a full calc.
4. Read the values, with either option:
   - **(a) JSON:** `GET .../worksheets/{name}/usedRange(valuesOnly=true)?$select=address,values,valueTypes,formulas` (and `text` if testing formats). `valueTypes` ∈ {Unknown, Empty, String, Integer, Double, Boolean, Error, RichValue}. Errors come back as strings in `values`, so use `valueTypes` to tell them apart. **Precision caveat:** doc examples show 15-significant-digit numbers (for example PMT → `5625.00000734125`), and I found no documentation guaranteeing round-trip doubles. **Verify empirically** with `=0.1+0.2`, `=1/3`, `=2^-1074`.
   - **(b) Exact:** with `persistChanges=true`, `closeSession`, wait or poll until the file's `lastModifiedDateTime`/eTag changes, then `GET .../content` and parse `<v>` with the *same* parser we use for corpora. This gives exact 17-digit values and a uniform pipeline. Persistence can lag by several seconds (Q&A above).
5. Delete the item, or keep it for audit. Cache the result by sha256.

**Latency/throughput (estimates, measure before relying on them):** roughly 1–3 s upload, 2–15 s session open and calc for small to medium files, and about 0.3–1 s per range read. That is about 5–30 s per workbook sequentially. Running 4–8 *different* workbooks in parallel stays inside throttling guidance, which gives very roughly 500–3,000 small workbooks per hour. Fine for cache-miss-only use. Too slow for "every PR re-oracles everything", which is why goldens get committed.

**Is web calc identical to desktop?** It is the same Excel calc engine family and ships the same M365 functions (dynamic arrays, LAMBDA, REGEX*, GROUPBY/PIVOTBY, and so on). In the browser UI, Microsoft's **Calc.ts** (a TypeScript engine derived from the Calc Intelligence team's F# semantics, *not* a port of the desktop engine) pre-evaluates about 99% of formulas client-side "with accuracy of 99.95%", and the server remains authoritative (https://www.microsoft.com/en-us/garage/wall-of-fame/calc-ts-in-excel-for-the-web/). Graph has no browser, so its results should come from the server engine. Known web differences:
- `FILTERXML` is "not available in Excel for the web and Excel for Mac" (https://support.microsoft.com/en-us/office/filterxml-function-4df72efc-11ec-4951-86f5-c1374812f5b7). `WEBSERVICE` is likewise unavailable on web and Mac.
- No VBA, UDFs or XLL/COM add-in functions. `CALL`/`REGISTER.ID` are unavailable. `RTD` cannot run (cached values only).
- `INFO` and parts of `CELL` (filename/directory) differ.
- External links cannot be updated.
- Data tables, Goal Seek and Solver are not available.
- Differences page: https://support.microsoft.com/en-us/office/differences-between-using-a-workbook-in-the-browser-and-in-excel-f0dc28ed-b85d-4e1d-be6d-5878005db3b6

For these cases, mark the oracle result `backend=graph`, keep a denylist of functions, and use the desktop backend.

**Auth for an unattended agent:**
- **Delegated plus device-code flow (recommended first).** Register a public-client app, enable "Allow public client flows", and request scopes `Files.ReadWrite offline_access User.Read`. The owner signs in once with a device code (and MFA). MSAL persists the token cache, which holds the refresh token. Refresh tokens last **90 days** for non-SPA public clients and "replace themselves with a fresh token upon every use", so the lifetime effectively slides while in use (https://learn.microsoft.com/en-us/entra/identity-platform/refresh-tokens). They are revoked by a password change by the user or an admin, "revoke sessions", or Conditional Access sign-in-frequency policies. Downside: the cache **rotates**, so a CI secret goes stale unless written back. Keep the cache on the oracle runner's disk, or in a key vault the job can update, rather than in a read-only GitHub secret.
- **App-only (client credentials).** Needs a work/school tenant, an app registration with a client secret or certificate, the **Application** permission `Files.ReadWrite.All` (or, better, `Sites.Selected` plus a per-site grant to a dedicated SharePoint site), and admin consent (the owner is admin of their own tenant). Call `/users/{upn}/drive/...` or `/drives/{driveId}/items/...`. Workbook endpoints are documented as unsupported for app-only but reported working. **Test it.** If it works, it is the cleanest unattended option: no rotation, and secrets fit in GitHub Actions. Whether workbook APIs work with `Sites.Selected` is also unverified.
- Avoid ROPC (username/password). It breaks with MFA and Microsoft discourages it.

**Office Scripts / Power Automate (B.3):** these need a business license for Power Automate. Personal/Family Office Scripts is preview-only via the Insider program. Limits: **1,600 "Run script" calls per user per day**, a **120 s** synchronous timeout, 5 MB Excel payloads, and no `fetch` from scripts run by flows (https://learn.microsoft.com/en-us/office/dev/scripts/testing/platform-limits). A flow could be "file dropped in OneDrive folder → run script (`workbook.getApplication().calculate(ExcelScript.CalculationType.fullRebuild)`, dump values as JSON) → write JSON back". It works, but it is slower, lower quota and harder to version than Graph. Use it only if Graph app registration is blocked by an admin.

## B.2 Windows VM + Excel desktop (COM)

- **Microsoft's stance:** "Microsoft does not currently recommend, and does not support, Automation of Microsoft Office applications from any unattended, non-interactive client application or component (including ASP, ASP.NET, DCOM, and NT Services)" (https://support.microsoft.com/topic/considerations-for-server-side-automation-of-office-48bcfe93-8a89-47f1-0bce-017433ad79e2). A licensed path exists via **Microsoft 365 for unattended RPA**, still "AS IS", with guidance on dialogs, identity, STA single-threading ("isolate them at the virtual machine level") and restart-on-hang (https://learn.microsoft.com/en-us/office/client-developer/integration/considerations-unattended-automation-office-microsoft-365-for-unattended-rpa).
- **Licensing:**
  - On Windows Server, RDS or Azure Virtual Desktop multi-session, M365 Apps needs **shared computer activation**. That is available with plans including M365 Apps for enterprise (E3/E5, Apps for enterprise) and **Business Premium** only; Business Standard does *not* include it (https://learn.microsoft.com/en-us/microsoft-365-apps/licensing-activation/overview-shared-computer-activation, https://www.microsoft.com/licensing/guidance/Microsoft-365-apps).
  - Simplest compliant setup: a **single-user Windows 11 machine or VM the owner controls** (a home PC, or a Hyper-V/Parallels VM on their hardware), signed in as the licensed user under the normal "install on up to 5 PCs" right, running Excel in an **interactive, auto-logged-on desktop session** (not session 0 or a service).
  - Windows 11 client VMs on Azure carry extra Windows licensing requirements (Windows Enterprise E3/E5 or VDA multitenant hosting rights). Check before choosing Azure.
  - GitHub-hosted Windows runners do not include Office, and activating it per run is impractical. Use a **self-hosted runner** started in the interactive session ("run as a console app", not as a service). Label it `excel-oracle`.
- **Automation recipe (Python + pywin32; xlwings works too):**
  - Create `win32com.client.DispatchEx("Excel.Application")` for a fresh process per worker. Set `Visible=False`, `DisplayAlerts=False`, `ScreenUpdating=False`, `EnableEvents=False`, `AskToUpdateLinks=False`, `AutomationSecurity=3` (msoAutomationSecurityForceDisable, so no macros), and `Calculation` = manual before opening.
  - `Workbooks.Open(path, UpdateLinks=0, ReadOnly=True, IgnoreReadOnlyRecommended=True, AddToMru=False, Notify=False, CorruptLoad=0)`, then `Application.CalculateFullRebuild()`.
  - Either `SaveAs(out.xlsx, FileFormat=51)` and parse `<v>` (uniform with the corpus pipeline and exact), or read `UsedRange.Value2`. `Value2` gives exact doubles, avoids the Currency/Date coercion of `.Value` (`.Value` turns currency-formatted cells into 4-decimal Currency), and returns errors as ints (for example -2146826281 = #DIV/0!). Also read `.Formula2` to see dynamic arrays and `.HasSpill`/`SpillingToRange`.
  - Set `fullCalcOnLoad` beforehand as an extra guard.
- **Stability:** one Excel process per worker (STA), with a watchdog per workbook (for example 60–120 s) that kills `EXCEL.EXE` by PID (`GetWindowThreadProcessId` on `Application.Hwnd`). Delete stray `~$` lock files. Disable Protected View and file-block settings for the oracle folder (Trust Center → Trusted Locations). Disable add-ins, startup prompts, sign-in nags and update prompts (Office policy or registry). Disable AutoRecover. Restart Excel every N workbooks and reboot nightly. Run inside a dedicated user profile and a working directory under the profile.
- **Throughput (estimate):** small workbooks take about 0.3–1 s each per process (open, full calc, save, close). With about 4–8 processes on an 8-core VM, that is roughly 10–30k small workbooks per hour. Big models are dominated by calc time. This is 10–50× Graph and has no quotas.
- **Value:** the only route to the desktop-only functions (FILTERXML, WEBSERVICE (needs network; mock), CELL/INFO desktop semantics, data tables `TABLE()`, XLL functions if ever needed), plus Excel-desktop performance baselines.

## B.4 macOS Excel (brief)

AppleScript or xlwings (appscript) work. The witanlabs harness is a production-tested example: chunked `osascript`, `set display alerts to false`, VBA disabled via `defaults write com.microsoft.Excel VisualBasicMacroExecutionState DisabledWithoutWarnings`, kill and retry on hangs, and files kept under the user's home directory because of the Excel sandbox. Mac Excel has the same M365 function set except Windows-only functions (FILTERXML, WEBSERVICE), and serializes OOXML slightly differently (calipers warns against Mac goldens for round-trip tests; values are fine). Good for an owner with a Mac and an M365 license.

## B.5 LibreOffice headless (weak stand-in)

`soffice --headless --convert-to xlsx` with recalculation forced on load (`OOXMLRecalcMode`=always) gives a free, Linux-native "second opinion". It matches about 96.7–98.9% of formula cells versus Excel on real corpora (witan). Known divergences:
- Different string↔number coercion settings and empty-string handling.
- Missing or partial newer Excel functions: dynamic-array edge cases, GROUPBY/PIVOTBY, REGEX* signatures, IMAGE, newer text and array functions on older LO builds.
- Different numerics in some statistical, financial and engineering functions, and a different "approximately equal" snapping.
- WEEKNUM/WORKDAY and locale differences, plus the 1900-02-29 serial handling.
- No `#SPILL!`/`#CALC!` parity in some cases.
- Iterative calc and circular-ref behavior differ.

Use it only for triage ("RustSheet and LO agree, Excel unknown") and label its results `approximate`. Never use it as a golden.

---

## B.6 Recommended oracle architecture

```
crates/oracle (or tools/oracle, Python OK)
  trait Backend { fn evaluate(xlsx: &[u8], opts) -> OracleResult }
  backends:
    cached-v      parse Excel-written <v> (provenance-gated)        free, offline
    graph         upload → session → FullRebuild → read/download    needs work/school tenant
    excel-com     HTTP worker on Windows self-hosted box            fastest/most faithful
    excel-mac     AppleScript worker (witan-style)                  optional
    libreoffice   soffice headless                                  approximate, never golden
  cache: ~/.cache/rustsheet-oracle/{sha256}.{backend}.json.zst
  CLI:   rustsheet-oracle eval --backend graph|excel-com|cached-v file.xlsx [--sheets ...]
         rustsheet-oracle fill-goldens tests/corpus/**/*.xlsx   (only cache misses hit the network)
```

**Golden JSON (committed under `tests/golden/{sha256}.json`):**
```json
{ "source_sha256": "...", "source_path": "tests/corpus/ironcalc/FINANCIAL/XIRR.xlsx",
  "oracle": {"backend": "excel-com", "excel_build": "16.0.19127.20000", "channel": "Current",
             "os": "Windows 11 24H2", "calc": "FullRebuild", "generated_at": "2026-10-01T12:00:00Z",
             "mock_now": "2026-01-01T00:00:00Z"},
  "settings": {"date1904": false, "iterate": false, "fullPrecision": true},
  "cells": { "Sheet1!B2": {"t":"n","v":"0.30000000000000004"},
             "Sheet1!B3": {"t":"e","v":"#DIV/0!"},
             "Sheet1!B4": {"t":"s","v":"abc"}, "Sheet1!B5": {"t":"b","v":true},
             "Sheet1!D1:D3": {"spill": true} },
  "excluded": {"Sheet1!C9": "volatile:NOW"} }
```
- Store numbers as **round-trip strings** (or add `"bits":"0x3FD3333333333334"`) so JSON tooling cannot lose precision.
- Record only formula cells and spill areas, to keep files small. Zstd-compress big ones and store them with Git LFS if the total exceeds about 50 MB.
- Key by **content hash**, so re-saving a file invalidates its golden. Record the Excel build so we can re-baseline when Excel changes behavior (for example after a monthly channel update). Keep a `known_excel_changes.md`.
- Test runner: `cargo test -p rustsheet-compat` reads xlsx and golden, recalculates, and diffs. It reports bit-exact %, within 1e-12 relative %, within 1e-9 %, mismatch lists by function, and a per-function pass-rate dashboard. Agents work fully offline. CI fails on regressions (the pass set shrinks), not on absolute rates.
- **Formula fuzzer → oracle loop:** generate a batch xlsx (thousands of formulas on a fixed input grid), send it to the oracle once, commit the golden, and hill-climb offline.
- Provenance gate for the `cached-v` backend: require an Excel app name, `calcMode != manual`, no `fullCalcOnLoad`, and recent `calcId` (bucket the rest). Exclude volatile cells.

---

## B.7 Owner checklist (Microsoft Graph backend)

**0. Account check.** Sign in at https://portal.office.com. If the account is `@outlook.com`/`@hotmail.com`/`@live.com`, or the plan is M365 Personal/Family, Graph's Excel API will not work. Get a work/school tenant: Microsoft 365 Business Basic is enough for Graph (Business Standard if you also want desktop Excel for the COM backend; Business Premium if you want shared computer activation on Windows Server or AVD). You become the tenant's Global Admin.

**1. Prepare storage.**
- Option A (simplest): use your own OneDrive for Business folder `/rustsheet-oracle/`.
- Option B (least privilege for app-only): create a SharePoint team site "RustSheet Oracle" with a document library `oracle`.

**2. Register the app** in the Entra admin center (https://entra.microsoft.com) → Identity → Applications → App registrations → New registration.
- Name: `rustsheet-oracle`. Supported account types: *Accounts in this organizational directory only*.
- Redirect URI: none, or "Public client/native (mobile & desktop)" `https://login.microsoftonline.com/common/oauth2/nativeclient`.
- Note the **Application (client) ID** and **Directory (tenant) ID**.

**3a. Delegated path (works per the docs):**
- Authentication → Advanced settings → **Allow public client flows = Yes**.
- API permissions → Microsoft Graph → Delegated: `Files.ReadWrite`, `offline_access`, `User.Read` → **Grant admin consent**.
- Run a one-time login on the oracle machine. A Python sketch with `msal.PublicClientApplication(client_id, authority=f"https://login.microsoftonline.com/{tenant}", token_cache=SerializableTokenCache())`, then `initiate_device_flow(scopes=["Files.ReadWrite"])`: open https://microsoft.com/devicelogin, enter the code, sign in with MFA.
- Persist the cache file (`~/.config/rustsheet-oracle/msal_cache.json`, mode 600). The tool calls `acquire_token_silent` and **writes the cache back after every refresh**. Run it at least weekly so the 90-day window never lapses. Re-run the device login after a password change or a token revocation.

**3b. App-only path (optional; officially unsupported for workbook endpoints, so test first):**
- Certificates & secrets → upload a certificate (preferred), or New client secret (expiry ≤ 12–24 months, with a calendar reminder).
- API permissions → Microsoft Graph → **Application**: `Files.ReadWrite.All` (broad) **or** `Sites.Selected` (narrow). For Sites.Selected, grant the app `write` on the oracle site via `POST /sites/{site-id}/permissions` with `{"roles":["write"],"grantedToIdentities":[{"application":{"id":"<client-id>","displayName":"rustsheet-oracle"}}]}`, using an admin token.
- **Grant admin consent.**
- Get the target drive: `GET /users/{you@tenant}/drive` or `GET /sites/{site-id}/drive` → `driveId`.
- Smoke test: upload a workbook containing `=0.1+0.2`, `=1/3`, `=FILTER(...)`, `=REGEXEXTRACT(...)`, `=XLOOKUP(...)`. Then createSession → calculate FullRebuild → usedRange → download. If any workbook call returns 401/403 under app-only, fall back to 3a.

**4. Store secrets (never paste them into chat):**
- Claude Code cloud environment: environment settings (session title bar → environment → Edit) → environment variables / API credentials:
  - `RUSTSHEET_ORACLE_TENANT_ID`, `RUSTSHEET_ORACLE_CLIENT_ID`
  - app-only: `RUSTSHEET_ORACLE_CLIENT_SECRET` (or `RUSTSHEET_ORACLE_CERT_PEM_B64`), `RUSTSHEET_ORACLE_DRIVE_ID`
  - delegated: `RUSTSHEET_ORACLE_MSAL_CACHE_B64` (bootstrap only; refreshed copies must be persisted elsewhere)
  - `RUSTSHEET_ORACLE_FOLDER=/rustsheet-oracle`
  - Allow network hosts `login.microsoftonline.com`, `graph.microsoft.com` and `*.sharepoint.com` (download redirects).
- GitHub Actions: repo → Settings → Secrets and variables → Actions → add the same names. Run oracle jobs only on `workflow_dispatch` or a schedule in a protected environment (`oracle`) with required reviewers, never on fork PRs.
- For the delegated cache in CI, prefer running the oracle job on the self-hosted runner that keeps the cache on disk. Alternatively, store it in Azure Key Vault or a private gist the job can update.

**5. Operational rules for the tool:** sequential per workbook, at most about 4–8 workbooks in flight. Honor `Retry-After`. Use `Prefer: respond-async` on createSession. Chunk range reads under 5 MB. Delete uploaded files after reading. Log `x-ms-ags-diagnostic`/`request-id` on failures. Cache everything by sha256.

**6. Optional desktop backend:** a Windows 11 box with the owner's M365 Apps (Business Standard or higher), auto-logon to a dedicated local user, and Excel signed in once. Install Python and pywin32, and register a GitHub self-hosted runner as a console app (not a service) with label `excel-oracle`. Add the oracle folder as a Trusted Location, disable Protected View and add-ins, and set up a nightly reboot.

---

## Sources

- IronCalc: https://github.com/ironcalc/IronCalc (local clone inspected)
- Formualizer: https://github.com/psu3d0/formualizer
- HyperFormula: https://github.com/handsontable/hyperformula (`test/fetch-tests.sh`, `docs/guide/compatibility-with-microsoft-excel.md`)
- LibreOffice function tests: https://github.com/LibreOffice/core/tree/master/sc/qa/unit/data/functions
- OpenFormula conformance (LO-derived): https://github.com/workspace-sh/table-file-format/pull/134
- Apache POI test data: https://github.com/apache/poi/tree/trunk/test-data/spreadsheet ; https://poi.apache.org/components/spreadsheet/eval-devguide.html
- Gnumeric: https://gitlab.gnome.org/GNOME/gnumeric (samples/excel, test/t10xx)
- xlcalculator: https://github.com/bradbase/xlcalculator ; pycel: https://github.com/dgorissen/pycel ; formulas: https://github.com/vinci1it2000/formulas ; formulajs: https://github.com/formulajs/formulajs
- witanlabs xlsx-corpus-bench: https://github.com/witanlabs/xlsx-corpus-bench
- calipers: https://github.com/fundamental-research-labs/calipers (PR #46: https://github.com/fundamental-research-labs/calipers/pull/46)
- SpreadsheetBench: https://github.com/RUCKBReasoning/SpreadsheetBench
- FUSE: https://zenodo.org/records/581678 ; https://www.barik.net/posts/fuse-a-reproducible-extendable-internet-scale-corpus-of-spreadsheets/
- Enron: https://figshare.com/articles/Enron_s_Spreadsheets_and_Related_Emails_A_Dataset_and_Analysis/1222882 ; Enron vs EUSES: https://arxiv.org/html/1503.04055
- VEnron: https://dl.acm.org/doi/10.1145/2889160.2889238
- OpenFormula spec: https://docs.oasis-open.org/office/v1.2/OpenDocument-v1.2-part2.html
- MS support example page (pattern): https://support.microsoft.com/en-us/office/product-function-8e6b5b24-90ee-4650-aeec-80982a0512ce
- MS template copyright: https://www.microsoft.com/en-us/legal/intellectualproperty/copyright/permissions
- Graph Excel overview: https://learn.microsoft.com/en-us/graph/api/resources/excel
- Graph calculate: https://learn.microsoft.com/en-us/graph/api/workbookapplication-calculate
- Graph usedRange: https://learn.microsoft.com/en-us/graph/api/worksheet-usedrange
- Graph best practices: https://learn.microsoft.com/en-us/graph/workbook-best-practice
- Graph error handling: https://learn.microsoft.com/en-us/graph/workbook-error-handling
- Graph throttling: https://learn.microsoft.com/en-us/graph/throttling
- App-only calculate Q&A: https://learn.microsoft.com/en-us/answers/questions/2143028/ms-graph-workbookapplication-calculate-api-with-ap
- Office add-in resource limits (5 MB / 5M cells): https://learn.microsoft.com/en-us/office/dev/add-ins/concepts/resource-limits-and-performance-optimization
- Excel for the web service description (100 MB): https://learn.microsoft.com/en-us/office365/servicedescriptions/office-online-service-description/excel-online
- Web vs desktop differences: https://support.microsoft.com/en-us/office/differences-between-using-a-workbook-in-the-browser-and-in-excel-f0dc28ed-b85d-4e1d-be6d-5878005db3b6
- FILTERXML availability: https://support.microsoft.com/en-us/office/filterxml-function-4df72efc-11ec-4951-86f5-c1374812f5b7
- Calc.ts: https://www.microsoft.com/en-us/garage/wall-of-fame/calc-ts-in-excel-for-the-web/
- Refresh tokens: https://learn.microsoft.com/en-us/entra/identity-platform/refresh-tokens
- M365 dev program eligibility: https://learn.microsoft.com/en-us/office/developer-program/join-with-visual-studio
- Office Scripts limits: https://learn.microsoft.com/en-us/office/dev/scripts/testing/platform-limits
- Server-side automation stance: https://support.microsoft.com/topic/considerations-for-server-side-automation-of-office-48bcfe93-8a89-47f1-0bce-017433ad79e2
- Unattended RPA considerations: https://learn.microsoft.com/en-us/office/client-developer/integration/considerations-unattended-automation-office-microsoft-365-for-unattended-rpa
- Shared computer activation: https://learn.microsoft.com/en-us/microsoft-365-apps/licensing-activation/overview-shared-computer-activation ; https://www.microsoft.com/licensing/guidance/Microsoft-365-apps

## Open questions to verify empirically (first oracle session)
1. Does Graph `range.values` return round-trip doubles or 15-digit-rounded numbers? Test `=0.1+0.2`, `=1/3`, `=PI()`. If they are rounded, use the download path for goldens.
2. Do workbook endpoints work app-only (`Files.ReadWrite.All`, and also `Sites.Selected`) in the owner's tenant?
3. After `calculate` plus `closeSession` with `persistChanges=true`, does the downloaded file always contain fresh `<v>` for every formula cell, including spills? How long is the persistence lag?
4. Do Graph and Excel desktop (same build/channel) agree on the whole IronCalc plus POI corpus? Expect yes, except for the denylisted functions. Run this once as a calibration.
