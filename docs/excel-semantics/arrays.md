# Arrays

## Cached `#CALC!` / `#SPILL!` are stored as `#VALUE!` plus a `vm` attribute
Rule: Excel writes a dynamic-array `#CALC!` (e.g. DROP/TAKE to nothing, `SEQUENCE(0,0)`, FILTER with no
match) or `#SPILL!` into the sheet XML as `t="e"` with `<v>#VALUE!</v>` and a `vm="N"` attribute. `vm`
indexes `xl/metadata.xml` `<valueMetadata>`; its `<rc v=K>` indexes `xl/richData/rdrichvalue.xml`. The
`errorType` field (13 = `#CALC!`, 8 = `#SPILL!`) must be found **by key name** in that value's
structure (`xl/richData/rdrichvaluestructure.xml`): desktop files put it first, but Excel for the web
writes the `_error` keys alphabetically (`colOffset, errorType, rwOffset, subType`), so there the
first field is `colOffset`. (The earlier "first field is the errorType" was wrong for those files;
T-0208, gen-m2/78bc88199fc96650 Calc!A61 rv `0 8 5 1` is `#SPILL!`.) A reader that ignores `vm`
sees `#VALUE!`. Other errorTypes exist (10 for an uncached external reference, references.md); until a
probe maps them (T-0030) the harness reads them as `#UNDECODED!` and never scores them.
Evidence: ironcalc-calc/ERROR.TYPE.xlsx (A8 vm→13, ERROR.TYPE=14; A14 vm→8, ERROR.TYPE=9);
DROP_TAKE.xlsx C2/E2/F2/I2/L2, FILTER_ERRORS.xlsx E1, SEQUENCE.xlsx C16/E18 (all errorType 13, subType 3).
Status: confirmed from Excel-cached files. (An earlier theory that these were CSE `#VALUE!` results was
wrong: the cells are `cm="1"` dynamic arrays and the engine's `#CALC!` was already right.)
Implemented in: `read_rich_errors` in `crates/rustsheet-eval/src/xlsx_snapshot.rs` (eval expected values).

## NOT and CONCATENATE lift over array arguments
Rule: `NOT(array)` and `CONCATENATE(array, …)` return an array, one result per element, like the
`&` operator and TRIM. They do not return a scalar `#VALUE!`. When the result cannot spill (the
next cell is occupied) the cell shows `#SPILL!`. `CONCAT`, by contrast, joins a whole range into
one string.
Evidence (oracle: excel-graph, 2026-10-04, gen-m2 sidecars as decoded by T-0208; each argument is a
multi-cell column from `INDEX(range,0,…)` and the row below is occupied, so the cell is `#SPILL!`):
gen-m2/75840b9c71a50596 Calc!A29 `=NOT(INDEX(Data!B2:E8,Data!C$10,3))` (C10 = FALSE);
gen-m2/f7e6d351c8538ef5 Calc!A106 `=NOT(INDEX(Data!B3:E9,(Data!B$10-0.1),4))` (B10 = 0.2);
gen-m2/207a5753da48c7e0 Calc!A297 `=(AVERAGE(…)=CONCATENATE(INDEX(Data!C11:C12,0),("#N/A"&Data!D4)))`;
gen-m2/b4392a6514a36ade Calc!A348 `=TRIM(CONCATENATE(INDEX(Data!A7:D11,Data!A8,3)))` (A8 = 0.3).
Status: VERIFIED for the array shape and #SPILL! (NOT returns an array). ASSUMED, pending probe T-0232: the per-element cast
(text, empty and error elements follow the scalar `cast_to_bool` rules); the oracle cells never showed an element value.
Implemented in: `fn_not` (`functions/logical/and_or_xor_not.rs`) and its `scalar_arguments` static-analysis entry (T-0225); CONCATENATE pending (T-0226).
Tasks: T-0225 (NOT), T-0226 (CONCATENATE)
