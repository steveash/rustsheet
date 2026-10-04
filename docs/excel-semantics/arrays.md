# Arrays

## Cached `#CALC!` / `#SPILL!` are stored as `#VALUE!` plus a `vm` attribute
Rule: Excel writes a dynamic-array `#CALC!` (e.g. DROP/TAKE to nothing, `SEQUENCE(0,0)`, FILTER with no
match) or `#SPILL!` into the sheet XML as `t="e"` with `<v>#VALUE!</v>` and a `vm="N"` attribute. `vm`
indexes `xl/metadata.xml` `<valueMetadata>`; its `<rc v=K>` indexes `xl/richData/rdrichvalue.xml`, whose
first field is the `errorType`: 13 = `#CALC!`, 8 = `#SPILL!`. A reader that ignores `vm` sees `#VALUE!`.
Evidence: ironcalc-calc/ERROR.TYPE.xlsx (A8 vm→13, ERROR.TYPE=14; A14 vm→8, ERROR.TYPE=9);
DROP_TAKE.xlsx C2/E2/F2/I2/L2, FILTER_ERRORS.xlsx E1, SEQUENCE.xlsx C16/E18 (all errorType 13, subType 3).
Status: confirmed from Excel-cached files. (An earlier theory that these were CSE `#VALUE!` results was
wrong: the cells are `cm="1"` dynamic arrays and the engine's `#CALC!` was already right.)
Implemented in: `read_rich_errors` in `crates/rustsheet-eval/src/xlsx_snapshot.rs` (eval expected values).
