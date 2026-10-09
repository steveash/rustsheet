# SPEC-0005 — Workbook date system (1900 / 1904)

- **Status:** draft
- **Milestone / lane:** M1 / engine (formatter: functions-date; import/export: engine)
- **Tasks:** T-1266 (this spec), T-1105 (first consumer)

## Problem
The engine has no notion of a workbook's date system. Every serial is read in the 1900 system,
the project's target (CLAUDE.md). Workbooks saved with `<workbookPr date1904="1"/>` (for example
poi's ElapsedFormatTests, cached by Mac Excel 16.0) are evaluated and formatted as if they were 1900
workbooks. This has two observable consequences today:

- TEXT and cell display of a negative serial under a format with an elapsed token (`[h]`, `[mm]`,
  `[s]`): Excel gives `#VALUE!` in the 1900 system (probes/text-negative-serial-elapsed-mixed
  A1, A3, A5, A6; dates.md, T-1079). In a 1904 workbook it keeps a leading minus (poi
  ElapsedFormatTests A40-A47; number-formats.md "Elapsed-time tokens"). The formatter
  (`ParsePart::Date` arm of `formatter/format.rs`) cannot tell the two apart. It exempts
  elapsed sections in every workbook, so the four 1900 probe cells fail (TODO(T-1105)).
- Every date component of a 1904 workbook (TEXT `yyyy`/`d`, YEAR, DATE, DATEVALUE, ...) is off by
  1462 days. The poi workbooks avoid most of this because their formats are time-only.
- Export drops `date1904`, so a 1904 workbook round-trips to a 1900 one and its date cells move.

## Goals / non-goals
Goals:
1. The date system is part of the workbook model: imported from xlsx, exported back, set for new
   workbooks through the facade (default 1900), and readable by the formatter and by functions.
2. Phase 1 consumer: the negative-serial rule in the formatter follows the date system (T-1105).
3. One place decides "is this serial a valid date in this workbook", so later consumers (date
   functions) reuse it instead of re-reading the flag.

Non-goals (separate tasks after this spec is accepted, each with its own probes):
- 1904 date arithmetic in date functions (DATE, YEAR, DATEVALUE, EDATE, WEEKDAY, financial date
  arguments) and the 1904 epoch in TEXT date tokens.
- Switching the date system of a loaded workbook (Excel's "Use 1904 date system" checkbox
  shifts no stored values; we only need the flag, not a conversion).
- 1904 behavior for anything an oracle cannot evidence. excel-graph evaluates an xlsx as stored,
  so it can answer once the case format can request 1904 (rollout step 2).

## Excel behavior to match
- 1900 system: a negative value under any date/time format, elapsed or not, is `#VALUE!` in TEXT
  (probes/text-ddd-negative-fraction A1-A9, probes/text-negative-serial-elapsed-mixed A1-A6).
- 1904 system, elapsed-only sections: the minus is kept and the elapsed magnitude printed
  (poi ElapsedFormatTests A40-A47, cached by Mac Excel 16.0).
- Unknown, to probe before implementing: 1904 with `d`/`mmm`/`yyyy` tokens on a negative
  (`TEXT(-1.5,"[h] d")`); `TEXT(-1E-10,"[h]:mm")` and `TEXT(-0,"[h]:mm")` in both systems; cell
  display (`#####` or text) of a negative under a date format in both systems. The case format
  can set neither the date system nor a cell number format, hence rollout step 2.

## Design
**Model.** Add to `WorkbookSettings` (crates/rustsheet-engine/src/types.rs):

```rust
/// The workbook's date system (`<workbookPr date1904>`); serial 0 is 1900-01-00 or 1904-01-01.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateSystem {
    #[default]
    Excel1900,
    Excel1904,
}

pub struct WorkbookSettings {
    pub tz: String,
    pub locale: String,
    pub date_system: DateSystem,
}
```

`WorkbookSettings` is part of the bitcode `Model::to_bytes` format, so an older blob does not
decode. No blob is committed or promised stable (check `rg from_bytes` at implementation time).
If one is, decode with a defaulted field instead.

**Import/export (rustsheet-xlsx).** `import/workbook.rs` reads `workbookPr@date1904` (`1`/`true`
→ `Excel1904`; absent, `0` or `false` → `Excel1900`; anything else → `Excel1900`, never an
error, since workbooks are untrusted). `export` writes `<workbookPr date1904="1"/>` only for
1904, so 1900 exports stay byte-identical.

**Formatter.** `format_number` and `format_text` gain the date system through a small options
value, not a bare bool, so later per-workbook format inputs have one home:

```rust
pub struct FormatContext<'a> { pub locale: &'a Locale, pub date_system: DateSystem }
```

The 12 current call sites pass `self.locale` today. Model call sites build the context from
`self.workbook.settings`; locale-only helpers (`number_format.rs`, examples in `model.rs`) use
`DateSystem::default()`. The `ParsePart::Date` arm becomes:
`if value < 0.0 && !(has_elapsed && date_system == Excel1904) { return invalid_date(value) }`.
The rule lives in one function, `formatter::dates::negative_serial_allowed(tokens, date_system)`,
cited by dates.md.

**Facade (crates/rustsheet).** `WorkbookOptions` gains `date_system` (default 1900) for new
workbooks, and `Workbook` exposes it read-only. Python bindings mirror both. The eval
harness needs no change for imported cases: it loads xlsx, so it gets the flag for free.

## Alternatives considered
- *Always `#VALUE!` (drop the elapsed exemption).* Matches the 1900 target in one line, but
  regresses poi ElapsedFormatTests A40-A47, which directive 2 forbids without a decision record.
  It also leaves export dropping `date1904`. Rejected.
- *A bool on `Model` only (not in `WorkbookSettings`).* It is lost by `to_bytes` and invisible to
  export. Rejected.
- *Thread `&Model` into the formatter.* Couples a pure module to the model. Rejected for the
  context value.

## Test plan
- Unit: import of `date1904="1"`, `"true"`, `"0"`, absent and garbage; export round trip; the
  formatter rule in both systems.
- Eval: probes/text-negative-serial-elapsed-mixed passes, and poi ElapsedFormatTests A40-A47 stays
  passing (poi must be fetched locally: `rustsheet-eval fetch --suite poi`).
- New probes (rollout step 2): the unknowns listed above, filled by excel-graph from 1904 xlsx cases.
- No benchmark: one field read per formatted date section.

## Rollout
1. (engine, S) `DateSystem` in `WorkbookSettings`, xlsx import/export of `date1904`, facade
   option and getter. No behavior change. Unit tests only.
2. (harness, S) Case format: an optional workbook `date_system: 1904` and per-cell number formats,
   written by the spec→xlsx builder. Probe the unknowns above.
3. (functions-date, S) `FormatContext` and `negative_serial_allowed`. T-1105 is done when
   probes/text-negative-serial-elapsed-mixed passes and poi A40-A47 does not regress.
4. Later, per evidence: 1904 epoch in date functions and TEXT date tokens (planner splits).
