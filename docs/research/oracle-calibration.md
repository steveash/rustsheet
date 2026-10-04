# Oracle calibration: Excel for the web vs desktop Excel's cached values

Task T-0004, 2026-10-04. Question: when the excel-graph oracle (Excel for the web via Microsoft
Graph) recalculates workbooks that desktop Excel already computed and saved, how often does it
agree with the values desktop Excel stored? This bounds how far the oracle can be trusted as
"Microsoft 365 Excel", and checks the cached values ironcalc-calc gates the engine on.

## Method

- Inputs: every workbook of suite `ironcalc-calc` (`crates/rustsheet-xlsx/tests/calc_tests`,
  210 files), each recalculated through the oracle with the embedded canary (every result proven
  recalculated, en-US locale pinned; docs/architecture/oracle.md).
- Comparison (every formula cell, including array-formula spill cells): `rustsheet-eval compare` (crates/rustsheet-eval/src/calibrate.rs), i.e. the eval
  harness's own rules (`value::compare`: bit-exact pass, `numeric_ulp` ≤ 4 ULPs, `numeric_close`
  ≤ 1e-9 relative, ...), volatile-cell skipping and function attribution.
- Like the suite (`require_excel_authored`), workbooks not saved by Excel are skipped: their
  cached values are not Excel's answers (e.g. GEOMEAN.xlsx stores errors as text and 10-digit
  numbers).
- Reproduce: `cd tools && uv run rustsheet-oracle calibrate -b excel-graph --suite ironcalc-calc`
  (about 10 minutes with `--jobs 4`; re-runs reuse `eval/.cache/oracle/`).

## Result

- Workbooks compared: 206; refused: 1; skipped as not saved by Excel: GEOMEAN.xlsx, XMATCH_arrays.xlsx, array_in_scalar_repro.xlsx
- Formula cells compared: 71765; agree bit-exact: 71736 (99.96%); volatile cells skipped: 3

| verdict | cells |
|---|---|
| pass | 71736 |
| numeric_ulp | 11 |
| type_mismatch | 9 |
| numeric_mismatch | 5 |
| numeric_close | 4 |

Functions with disagreements (a cell counts once for each function it calls):

| function | cells | disagree | verdicts |
|---|---|---|---|
| (array member) | 1204 | 3 | numeric_ulp 3 |
| IMARGUMENT | 24 | 3 | numeric_ulp 3 |
| IMREAL | 417 | 3 | numeric_close 1, numeric_mismatch 2 |
| INDIRECT | 174 | 3 | type_mismatch 3 |
| ATAN2 | 27 | 2 | numeric_ulp 2 |
| IMSQRT | 52 | 2 | numeric_mismatch 2 |
| VALUE | 33 | 2 | numeric_mismatch 2 |
| (operators only) | 1745 | 1 | type_mismatch 1 |
| BESSELJ | 119 | 1 | numeric_close 1 |
| BESSELY | 119 | 1 | numeric_close 1 |
| COLUMN | 58 | 1 | type_mismatch 1 |
| DATEVALUE | 60 | 1 | type_mismatch 1 |
| DAYS | 76 | 1 | type_mismatch 1 |
| DAYS360 | 100 | 1 | type_mismatch 1 |
| ERFC | 19 | 1 | numeric_ulp 1 |
| ERFC.PRECISE | 19 | 1 | numeric_ulp 1 |
| IMAGINARY | 417 | 1 | numeric_close 1 |
| IMCOS | 48 | 1 | numeric_close 1 |
| IMTAN | 52 | 1 | numeric_close 1 |
| IRR | 91 | 1 | numeric_mismatch 1 |
| MIRR | 224 | 1 | numeric_ulp 1 |
| NETWORKDAYS | 76 | 1 | type_mismatch 1 |
| NETWORKDAYS.INTL | 104 | 1 | type_mismatch 1 |
| ROW | 62 | 1 | type_mismatch 1 |

Functions agreeing on every cell: 397.

Refused workbooks:

- `SHEET_SHEETS.xlsx`: refusing to recalculate: xl/worksheets/sheet1.xml uses SHEETS() or INFO(), whose answer the canary sheet would change

## What the 29 disagreements are

| group | cells | functions | who is right for en-US Microsoft 365 |
|---|---|---|---|
| Day-first date text in the workbooks' authoring Excel | 7 | DATEVALUE, VALUE (2), DAYS, DAYS360, NETWORKDAYS, NETWORKDAYS.INTL | **the oracle** |
| `INDIRECT` to a sheet that does not exist | 4 | INDIRECT, its dependant `=A5`, ROW(INDIRECT(…)), COLUMN(INDIRECT(…)) | **desktop Excel** (#REF!) |
| Last-digit floating point / near-zero residue | 18 | ATAN2 (2), IMARGUMENT (3), IMREAL∘IMSQRT (2), IMCOS, IMTAN, ERFC + ERFC.PRECISE (2), BESSELJ, BESSELY, MIRR, IRR, spilled SIN(4) (3 array cells) | both are Excel |

1. **Locale of the ironcalc authoring Excel.** `DATEVALUE("26-01-01")` is cached as 36917
   (26 Jan 2001) and `VALUE("2/3/2023")` as 44987 (2 Mar); "July 1, 2026" is cached as #VALUE!
   in DAYS/DAYS360/NETWORKDAYS. The oracle, pinned to en-US by the canary, reads month-first
   (#VALUE!, 44960 = 3 Feb) and parses month names (30, 30, 23). The workbooks were saved by a
   day-first Excel (desktop and Excel Online alike), so **ironcalc-calc's cached values are not
   en-US evidence for date text**. Committed en-US evidence: probes/date-text-locale
   (docs/excel-semantics/dates.md). The engine agrees with en-US for VALUE already (so ironcalc
   `C15`/`I5` are "known failures" where the engine is right) but reads `"26-01-01"` day-first
   and rejects month names in date arguments, passing those ironcalc cells only because the
   cached values share the mistake: T-0024.
2. **Missing-sheet references.** `INDIRECT("Sheet7!R2C2",FALSE)` and
   `ROW/COLUMN(INDIRECT("NonExist!W3"))` are #REF! in desktop (Mac) Excel; Excel for the web
   treats the missing sheet as an empty one (0, 3, 23). A direct probe confirmed it (`ISREF` is
   TRUE). The engine follows desktop; the oracle must not be used as evidence here
   (docs/excel-semantics/lookup.md).
3. **Floating point.** 11 cells are within 4 ULPs (including spilled `SIN(4)`: desktop
   -0.7568024953079283, oracle -0.7568024953079282, the correctly rounded value), 4 within 1e-9 relative, and 3 are residues
   around 1e-16 where one side's rounding leaves a tiny value (`IMREAL(IMSQRT(-x))` 2.121e-16
   vs 2.122e-16; two-flow IRR 2.2e-16 vs 0). Excel for the web and desktop evidently use
   slightly different math libraries or evaluation orders. Oracle goldens for these functions
   may therefore disagree with desktop-Excel-cached values by a few ULPs; eval scores both
   strictly, so cite which one a golden comes from.

Everything else (397 functions, 71,736 cells, including every financial, statistical, text,
lookup and date function in the suite beyond the cells above) agrees bit-exact.

## Refused and skipped

- `SHEET_SHEETS.xlsx`: refused, because it uses SHEETS()/INFO(), whose answers the canary sheet
  would change.
- `GEOMEAN.xlsx`, `XMATCH_arrays.xlsx`, `array_in_scalar_repro.xlsx`: not saved by Excel,
  skipped (also unscored by eval).

## All disagreeing cells

Shared-formula members show their group's master formula text (snapshot convention), so the
formula column can name a neighbouring input; the analysis above uses the real inputs.

| workbook | cell | formula | Excel cached | oracle | verdict |
|---|---|---|---|---|---|
| ATAN2_POWER.xlsx | Sheet1!D14 | `=ATAN2(A3,B3)` | 1.2626272556789115 | 1.2626272556789118 | numeric_ulp |
| ATAN2_POWER.xlsx | Sheet1!D26 | `=ATAN2(A26,B26)` | 1.1071487177940906 | 1.1071487177940904 | numeric_ulp |
| BESSEL.xlsx | All!D18 | `=BESSELJ(A3,B3)` | 0.001846551246313725 | 0.0018465512463137278 | numeric_close |
| BESSEL.xlsx | All!F18 | `=BESSELY(A3,B3)` | 0.001719431947654535 | 0.0017194319476545319 | numeric_close |
| COMPLEXs.xlsx | Sheet1!AL12 | `=IMREAL(IMSQRT(D3))` | 2.12115047744981e-16 | 2.12201937071624e-16 | numeric_mismatch |
| COMPLEXs.xlsx | Sheet1!AL26 | `=IMREAL(IMSQRT(D3))` | 6.12323399573677e-17 | 6.1257422745431e-17 | numeric_mismatch |
| COMPLEXs.xlsx | Sheet1!H10 | `=IMARGUMENT(D3)` | 1.1071487177940906 | 1.1071487177940904 | numeric_ulp |
| COMPLEXs.xlsx | Sheet1!H2 | `=IMARGUMENT(D2)` | 0.46364760900080615 | 0.4636476090008061 | numeric_ulp |
| COMPLEXs.xlsx | Sheet1!H3 | `=IMARGUMENT(D3)` | -1.1071487177940906 | -1.1071487177940904 | numeric_ulp |
| COMPLEXs.xlsx | Sheet1!K25 | `=IMAGINARY(IMCOS(D3))` | 7.58155274274655 | 7.58155274274654 | numeric_close |
| COMPLEXs.xlsx | Sheet1!N6 | `=IMREAL(_xlfn.IMTAN(D3))` | -3.38051500624658 | -3.38051500624659 | numeric_close |
| DATEVALUE.xlsx | DATEVALUE!B48 | `=DATEVALUE(A37)` | 36917.0 | '#VALUE!' | type_mismatch |
| DAYS_DAYS360.xlsx | Sheet1!C71 | `=_xlfn.DAYS(A71,B71)` | '#VALUE!' | 30.0 | type_mismatch |
| DAYS_DAYS360.xlsx | Sheet1!D71 | `=DAYS360(B71,A71)` | '#VALUE!' | 30.0 | type_mismatch |
| ERFs.xlsx | Sheet1!E8 | `=ERFC(A3)` | 2.1519736712498925e-17 | 2.151973671249892e-17 | numeric_ulp |
| ERFs.xlsx | Sheet1!G8 | `=_xlfn.ERFC.PRECISE(A3)` | 2.1519736712498925e-17 | 2.151973671249892e-17 | numeric_ulp |
| IF_ARRAY.xlsx | Sheet1!C11 | (array member) | -0.7568024953079283 | -0.7568024953079282 | numeric_ulp |
| IF_ARRAY.xlsx | Sheet1!C4 | (array member) | -0.7568024953079283 | -0.7568024953079282 | numeric_ulp |
| IF_ARRAY.xlsx | Sheet1!E4 | (array member) | -0.7568024953079283 | -0.7568024953079282 | numeric_ulp |
| INDIRECT.xlsx | r1c1_in_other_sheets!A5 | `=INDIRECT("Sheet7!R2C2", FALSE)` | '#REF!' | 0.0 | type_mismatch |
| INDIRECT.xlsx | r1c1_in_other_sheets!L6 | `=A5` | '#REF!' | 0.0 | type_mismatch |
| IRR.xlsx | Two Values!C4 | `=IRR(A3:B3)` | 2.220446049250313e-16 | 0.0 | numeric_mismatch |
| MIRR.xlsx | Sheet2!N16 | `=MIRR(A3:F3,J3,K3)` | 0.3933280241536672 | 0.3933280241536674 | numeric_ulp |
| NETWORKDAYS_NETWORKDAYS.INTL.xlsx | Sheet1!C60 | `=NETWORKDAYS(B60,A60)` | '#VALUE!' | 23.0 | type_mismatch |
| NETWORKDAYS_NETWORKDAYS.INTL.xlsx | Sheet1!D60 | `=NETWORKDAYS.INTL(B60,A60)` | '#VALUE!' | 23.0 | type_mismatch |
| ROW_COLUM.xlsx | ROW_COLUMN!K7 | `=ROW(INDIRECT(J3))` | '#REF!' | 3.0 | type_mismatch |
| ROW_COLUM.xlsx | ROW_COLUMN!L7 | `=COLUMN(INDIRECT(J3))` | '#REF!' | 23.0 | type_mismatch |
| T_VALUE_VALUETOTEXT.xlsx | Sheet1!C15 | `=VALUE(A15)` | 44987.0 | 44960.0 | numeric_mismatch |
| T_VALUE_VALUETOTEXT.xlsx | Sheet1!I5 | `=VALUE(H3)` | 36504.0 | 36445.0 | numeric_mismatch |
