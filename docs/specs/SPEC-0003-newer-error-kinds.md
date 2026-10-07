# SPEC-0003 — Excel's newer error kinds (#GETTING_DATA, #CONNECT!, #BLOCKED!, #FIELD!)

- **Status:** accepted (on merge of T-0177)
- **Milestone / lane:** M1 / api (implementation in engine, harness, research, functions-logic)
- **Tasks:** T-0177 (this spec); rollout tasks below

## Problem
The engine knows twelve errors: the classic eight (`#NULL!` … `#N/A`), plus `#SPILL!` and
`#CALC!`, plus the engine-only `#ERROR!`, `#N/IMPL` (its `Display`; the parsed literal is `#N/IMPL!`) and `#CIRC!`
(`crates/rustsheet-engine/src/expressions/token.rs`, `enum Error`). Excel has more. Without a kind for
them:
- `probes/error-literals-new` fails 17 of 18 cells, because `=#GETTING_DATA` does not parse and
  becomes `#ERROR!`.
- `probes/rich-errors-image`, `-field` and `-stockhistory` fail 13 cells. IMAGE, FIELDVALUE and
  STOCKHISTORY are `#NAME?`, so ERROR.TYPE and VALUETOTEXT of them are wrong too.
- The importer reads a cached `#BLOCKED!`, `#CONNECT!` or `#FIELD!` as `#VALUE!`
  (`value_metadata_errors`: "errors the engine has no kind for stay `#VALUE!`").
- T-0124 (an uncached external reference is `#BLOCKED!`) has no value to return.

These kinds touch the public API (`rustsheet::ErrorKind`, Python `XlError`), the literal tables,
ERROR.TYPE, xlsx import/export and the harness. That is why this design comes before any code
(CLAUDE.md directive 4).

## Goals / non-goals
Goals:
- One engine variant per newer error that has Excel evidence, with its ERROR.TYPE code, its
  literal, and its stored form in xlsx.
- Parsing that follows Excel: only the literals Excel accepts in formula text parse.
- Import of both stored forms (vm-coded rich values and plain `t="e"`), export that Excel reads
  back, and harness scoring against Excel's stored form.
- Facade and Python exposure, kept in sync by the existing `literals_match_engine` test.
- A defined headless result for IMAGE, STOCKHISTORY and FIELDVALUE (decision 0027 keeps their
  values out of scope, but their *error* is in scope).

Non-goals:
- `#BUSY!`, `#UNKNOWN!`, `#PYTHON!`, `#EXTERNAL!` and any other code. No formula produced them in a
  saved workbook, and their literals are rejected (arrays.md, syntax.md "Newer error literals").
  `ErrorKind` is `#[non_exhaustive]`, so they can be added later without a breaking change.
- Localized error names. The engine is en-US (decision 0030).
- Linked data types, images as cell values, and stock data (decision 0027).

## Excel behavior to match
| Kind | ERROR.TYPE | Formula literal? | How Excel stores it in xlsx | Evidence |
|---|---|---|---|---|
| `#GETTING_DATA` | 8 | **yes**, case-insensitive (`=#getting_data` too) | `t="e"`, `<v>#N/A</v>`, no `vm`: the saved file says `#N/A` | syntax.md "Newer error literals"; arrays.md table; probes/error-literals-new S!A1, A2, A11 |
| `#CONNECT!` | 10 | no (workbook rejected) | `t="e"`, `<v>#VALUE!</v>` plus `vm` → rich value errorType 9 | arrays.md table; probes/rich-errors-stockhistory A1–E1 |
| `#BLOCKED!` | 11 | no (workbook rejected) | `#VALUE!` plus `vm` → errorType 10 | arrays.md table; probes/rich-errors-image; references.md "External reference with no cached data is `#BLOCKED!`" |
| `#FIELD!` | 13 | no (workbook rejected) | `#VALUE!` plus `vm` → errorType 12 | arrays.md table; probes/rich-errors-field |

Behavior shared by all four:
- **Ordinary error kinds.** They propagate like any error. The leftmost error operand wins for
  `=`, `+`, `-`, `*` and `SUM` (syntax.md, T-0195: `#GETTING_DATA+#DIV/0!` is 8 and
  `#DIV/0!+#GETTING_DATA` is 2). For `&` and comparisons the rule is arrays.md "Errors broadcast
  through `&`, comparisons, POWER and the element-wise functions". No new precedence rule is needed.
- **None of them is `#N/A`.** For `#GETTING_DATA`: `ISNA` is FALSE, `ISERR` TRUE, `ISERROR` TRUE,
  `IFNA` passes it through, `IFERROR` catches it, and `TYPE` is 16 (error-literals-new A3, A4, A12,
  A15, A5, A16). For the three rich kinds, ISERR is TRUE, ISNA FALSE and TYPE 16 too, and
  `IFNA` passes them through (information.md, probes/rich-errors-args A40:B68, T-1012).
- **VALUETOTEXT and ARRAYTOTEXT** give the literal text (`"#GETTING_DATA"`, `"#BLOCKED!"`, …;
  error-literals-new A6, rich-errors-* E1/D1).
- **Stored constants (T-0999, syntax.md).** Excel for the web **rejects** a workbook that stores
  `<c t="e"><v>#BLOCKED!</v></c>`, `#CONNECT!`, `#FIELD!`, `#UNKNOWN!` or `#CALC!` as a literal
  (501 `unsupportedWorkbook`). It loads stored `#GETTING_DATA`, `#N/A` and `#BUSY!`. A stored `#BUSY!`
  is an error with `ERROR.TYPE` 0; it stays a non-goal here (follow-up T-1020).
- **Formula literals.** The other names (`=#BLOCKED!`, `=#CONNECT!`, `=#FIELD!`, and `#BUSY!`,
  `#UNKNOWN!`, `#PYTHON!`, `#EXTERNAL!`) make Excel for the web reject the whole workbook. They are
  not formula grammar.
- **Unknown, need evidence:**
  - what typing `#BLOCKED!` as cell input gives (T-1016).

The functions that produce them, as Excel evaluates them (oracle: excel-graph):
- **IMAGE** of an unreachable URL is `#BLOCKED!` (subType 12). It fetches the URL otherwise.
- **STOCKHISTORY** of an unknown ticker is `#CONNECT!` (subType 4). It calls a data service
  otherwise.
- **FIELDVALUE** of a plain value (not a linked data type) is `#FIELD!`.
- **An external-workbook reference with no cached data** is `#BLOCKED!` (subType 2; T-0124).

## Design

### Engine (`rustsheet-engine`, token.rs)
1. **New `Error` variants: `GETTINGDATA`, `CONNECT`, `BLOCKED`, `FIELD`.**
   - Append them **after** `NULL`. The enum derives `Serialize` and `bitcode::Encode`, which
     serialize by variant index. Appending keeps every existing index, and with them any encoded
     model.
   - `Display`, `to_localized_error_string` and `error_type_code` gain one arm each
     (8 / 10 / 11 / 13). Every `match` on `Error` stays exhaustive (no `_` arm is added), so the
     compiler finds every site.
   - `error_type_code` stays the single ERROR.TYPE table. Replace its "not produced" comment with
     the remaining unproduced codes, 12 (`#UNKNOWN!`) and 19 (`#EXTERNAL!`), kept as "no
     evidence of a producer".
2. **Two name tables, kept apart on purpose.**
   - **Stored names** (`get_error_by_english_name`): all sixteen names.
     The xlsx importer (`t="e"` values and rich-value names), the eval snapshot reader, and the
     facade's `ErrorKind::from_literal` use this table. `from_literal` moves to it from
     `get_error_by_name`, uppercasing as today. This lets Python `XlError("#BLOCKED!")` build the
     value.
   - **Formula and input names** (`get_error_by_name` with a `Language`, and the lexer's
     `consume_error`): the current twelve, plus `#GETTING_DATA` only. The lexer matches it as the
     en-US constant `"#GETTING_DATA"`, case-insensitively (evidence A11). It is not added to
     `Language::Errors`, because `language.bin` is a bitcode blob and a new field would break its
     decoding (non-goal: localization). `=#BLOCKED!` and the other names stay an unparseable
     formula, as today, which matches Excel refusing them.
   - Typed cell input (`Model::set_user_input` → `get_error_by_name`) and `value_needs_quoting`
     keep today's table until there is evidence of what Excel does with typed `#BLOCKED!` (desktop UI; not reachable by the web oracle). With that table, a
     typed `#BLOCKED!` stays text, and a typed `#GETTING_DATA` becomes the error because it is in
     the formula table. That choice **needs evidence** from desktop Excel (T-1016) and is recorded as UNVERIFIED in
     syntax.md.
   - **Criteria** (`is_english_error_string`, used by the COUNTIF/SUMIF family in
     `functions/criteria.rs`) keep **today's twelve names, unchanged**. Adding a name there would
     turn a criterion such as `"#BLOCKED!"` or `"<>#CONNECT!"` from text into an error match, and
     T-1012 probed it: COUNTIF with `"#BLOCKED!"`, `"#CONNECT!"` and `"#FIELD!"` counts 0 even
     over a cell holding that error (information.md), so the criteria table stays unchanged.
3. **Stored form: `Error::stored_literal(&self) -> &'static str`**, the single place that knows
   Excel saves `#GETTING_DATA` as `#N/A`. It returns `"#N/A"` for `GETTINGDATA` and the `Display`
   text otherwise. The xlsx exporter and the eval harness both call it (see below). It is a
   storage rule, not a value rule: in memory the value stays `GETTINGDATA`, so `ERROR.TYPE` is 8
   and `ISNA` FALSE. It is Excel's *logical* stored value, as the snapshot reader decodes it (with `vm`
   followed), so the rich kinds keep their names here. How our exporter writes them while it
   cannot yet emit `vm` is an exporter detail (below), not part of this rule.

### xlsx (`rustsheet-xlsx`)
- **Import:**
  - `rich_error_name` already maps errorTypes 8/9/10/12/13 to names.
  - The `(Some("#VALUE!"), Some(real)) if get_error_by_english_name(real).is_some()` guard in
    `import/worksheets.rs` then keeps the real kind once the stored table has it. No new code
    path: the "stay `#VALUE!`" sentence in arrays.md and in `value_metadata_errors`' doc changes to
    "unmapped errorTypes stay `#VALUE!`".
  - Plain `t="e"` cells with a new name (written by us, or by other tools) import through the same
    table.
- **Export:**
  - Error cells are written with `stored_literal()`, so `#GETTING_DATA` is saved as `#N/A`, as
    Excel does.
  - `#CONNECT!`, `#BLOCKED!` and `#FIELD!` must **not** be written as their literal: T-0999 shows that
    Excel rejects such a workbook. The same applies to `#CALC!`, which the exporter writes as a
    literal today, so that is an existing bug. `#SPILL!` stored as a literal is **unprobed**; it
    is treated the same way, and T-1014's oracle check confirms it.
  - The target is Excel's own form: `#VALUE!` plus `vm` and the rich-value parts (rollout X2,
    T-1014), for all five rich kinds (`#SPILL!`, `#CALC!`, `#CONNECT!`, `#BLOCKED!`, `#FIELD!`).
  - Until X2 lands, X1 adds one exporter-only helper in rustsheet-xlsx that writes those five as
    plain `#VALUE!`. That form loads in Excel and is what Excel's own file holds without `vm`.
    T-1014 deletes the helper. The kind is lost on a round trip through xlsx until then, now also
    for `#CALC!`/`#SPILL!`. That is a deliberate interim regression, and T-1011's PR calls it
    out.

### Harness (`rustsheet-eval`)
- Expected values come from Excel's saved files, decoded through `vm`. A `#GETTING_DATA` cell
  is therefore expected as `#N/A`, and a rich error as its real name (`#BLOCKED!`).
- The engine-side value of an error cell is compared through `Error::stored_literal()`: the
  same logical rule the exporter starts from, never a second mapping in the harness. The
  exporter-only `#VALUE!` fallback never applies to scoring.
- Rich-value names already decode to `#BLOCKED!`/`#CONNECT!`/`#FIELD!`, and the engine's
  `Display` matches them.

### Facade and Python
- **`ErrorKind`** gains `GettingData`, `Connect`, `Blocked` and `Field`:
  - with doc comments naming the literal;
  - `from_engine`/`to_engine` arms;
  - `literal()` arms;
  - `code()` stays `Some` for all of them.
- **Sync test:** `tests::literals_match_engine` already iterates over every kind. It gets the four
  new kinds, plus a check that `from_literal(kind.literal())` round-trips for all sixteen.
- **Naming clash:** the existing `ErrorKind::Unknown` means the engine's `#ERROR!`, not Excel's
  `#UNKNOWN!` (its doc says so). If `#UNKNOWN!` ever gets evidence, its variant is named
  `ExcelUnknown`. Renaming `Unknown` is a breaking API change and is not part of this spec.
- **Python:** `XlError` is built on `ErrorKind::from_literal`/`literal`/`code`, so it gains the
  kinds with no new code. One test per kind checks `XlError("#BLOCKED!").code == 11`, plus
  equality, and that a `=#GETTING_DATA` cell reads back as `XlError("#GETTING_DATA")`.

### Functions with no I/O (directive 5)
All three keep their Excel arity, and the result is a scalar error, never a spill, except FIELDVALUE with an
array first argument (below). Arguments are evaluated first. An argument error wins for IMAGE and STOCKHISTORY and for FIELDVALUE's `field_name`, but not for
FIELDVALUE's first argument (`FIELDVALUE(1/0,"Price")` is `#FIELD!`); an IMAGE source that is empty or a number is `#VALUE!`; an
array first argument to FIELDVALUE gives an array result (information.md, T-1012). Their behavior:
- **IMAGE(source, [alt_text], [sizing], [height], [width])** → `#BLOCKED!`.
  - Excel fetches `source` and returns an image value. We never fetch, so every IMAGE is
    `#BLOCKED!`.
  - This matches Excel for unreachable URLs (rich-errors-image A1). It is a **DEVIATION
    (decision 0014)** for reachable ones: no I/O from formulas.
  - It gets an `information.md` entry with `Status: DEVIATION (decision 0014)`. No eval cell
    exercises a reachable URL, so it needs no `deviations` line (decision 0036).
- **STOCKHISTORY(stock, start_date, [end_date], [interval], [headers], [property0], …)** →
  `#CONNECT!`.
  - This matches Excel for an unknown ticker (rich-errors-stockhistory A1). Every real ticker is a
    **DEVIATION (decision 0014)**, recorded in the same way.
  - When Excel itself returns `#CONNECT!` rather than `#BLOCKED!` is not known beyond that one
    cell. `#CONNECT!` is chosen because it is the only evidenced result.
- **FIELDVALUE(value, field_name)** → `#FIELD!`; an array `value` gives an array of `#FIELD!` (T-1012).
  - RustSheet has no linked data types (decision 0027), so every value is a plain value, and
    Excel gives `#FIELD!` for those (rich-errors-field A1). This is **not** a deviation.
- **External references** with no cached data → `#BLOCKED!`. T-0124 implements this on top of
  the new variant. External references are already a directive-5 no-I/O case.

## Alternatives considered
- **Map the newer errors onto `#VALUE!` (today) or `#N/A`.** This is wrong for ERROR.TYPE,
  VALUETOTEXT, ISNA and IFNA, against oracle evidence. Rejected.
- **One `Error::Other(u8)` variant carrying the ERROR.TYPE code.** It keeps the enum small, but
  every match then needs code-based logic. It also hides the kinds from exhaustive matching, and
  the facade would need per-code names anyway. Rejected: the set is small and evidenced.
- **Add the names to `Language::Errors`.** This would make them parse (wrong for three of the four)
  and needs a `language.bin` rebuild. Rejected; `#GETTING_DATA` is a constant in the lexer.
- **Store `#GETTING_DATA` as its own name on export.** Excel itself writes `#N/A`, and other
  readers know `#N/A`. Rejected; `stored_literal` follows Excel.
- **Write the rich kinds as their literal on export.** Excel rejects such a workbook (T-0999).
  Rejected.
- **Make IMAGE/STOCKHISTORY `#N/IMPL!` or `#NAME?`.** Both are wrong for the evidenced cells, and
  a parity engine should give Excel's offline answer. Rejected.

## Test plan
- Unit tests in token.rs for every new variant (and that `is_english_error_string` is unchanged):
  - its Display text and code;
  - the stored-name round trip;
  - that `#GETTING_DATA` (any case) lexes and that `#BLOCKED!` does not;
  - that `stored_literal` is `#N/A` for it.
- Facade: `literals_match_engine`, extended. Python: one round-trip test per kind.
- xlsx:
  - a round trip writes a model holding all four kinds and reads them back. `#GETTING_DATA` comes
    back as `#N/A`, matching Excel's own file. Until X2, the five rich kinds (via the exporter-only fallback) come back as `#VALUE!`.
  - The importer reads the committed rich-errors-* oracle workbooks as the real kinds
    (`value_metadata_errors` tests).
- Eval: `probes/error-literals-new` (18 cells) and `error-literals-new2` (15) pass, and the 13
  rich-errors-* known failures pass. Everything is recorded with `make eval-baseline`, with no
  regressions.
- Probe R1 (T-1012, done) added the evidence once marked "needs evidence": see information.md and
  probes/rich-errors-args. F1 implements the functions from it.

## Rollout
| # | Task | Lane | Size | Tier | Depends on |
|---|---|---|---|---|---|
| E1 | T-1010: Engine and facade `Error`/`ErrorKind` variants for `#GETTING_DATA`, `#CONNECT!`, `#BLOCKED!`, `#FIELD!` (codes, the two name tables, lexing `#GETTING_DATA`, `stored_literal`, Python tests) | engine | M | opus | — |
| X1 | T-1011: xlsx export and eval harness use `Error::stored_literal`; import keeps rich-value kinds (doc updates in arrays.md and rich_errors.rs) | harness | S | sonnet | E1 |
| R1 | T-1012: Probe IMAGE/STOCKHISTORY/FIELDVALUE argument errors, ISERR/ISNA/TYPE of rich errors, and COUNTIF criteria naming them | research | S | sonnet | — |
| F1 | T-1013: IMAGE, STOCKHISTORY, FIELDVALUE return their offline errors (DEVIATION entries for IMAGE and STOCKHISTORY) | functions-logic | S | sonnet | E1, R1 |
| X2 | T-1014: Export rich-value errors (`#SPILL!`, `#CALC!`, and the three new kinds) as `#VALUE!` plus `vm` metadata; T-0999 showed that stored literals make Excel reject the workbook | harness | M | opus | X1 |
| R2 | T-1016: Desktop probe of typed `#BLOCKED!`/`#GETTING_DATA` cell input (needs the excel-desktop oracle) | research | S | sonnet | — |
| B1 | T-1020: Stored `#BUSY!` constant (ERROR.TYPE 0): decide its representation | engine | S | opus | E1 |
| — | T-0124 (existing): uncached external reference is `#BLOCKED!` | engine | S | — | E1 |

Nothing stays behind an old path: each task replaces behavior in place. Until E1 lands, cells
of the new kinds keep failing as known failures.
