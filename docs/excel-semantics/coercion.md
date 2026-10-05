# Coercion and argument handling

## AND / OR evaluate every argument; an error anywhere wins
Rule: AND/OR do not short-circuit: `=OR(TRUE,(2<COUNTIFS(…shape mismatch…)))` → `#VALUE!`.
Evidence: gen-m2/78bc88199fc96650 Calc!A217. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0215

## MIN / MAX of a non-numeric text value argument is #VALUE!
Rule: a text value passed directly (not in a range) that is not a number makes MIN/MAX `#VALUE!`:
`=MAX(IFERROR(IF(Data!D4,"ü"),Data!$E$7),Data!D7:D9)` → `#VALUE!`.
Evidence: gen-m2/f7e6d351c8538ef5 Calc!A14. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0217

## LEFT / RIGHT / MID numeric arguments
Rule: booleans coerce (TRUE = 1: `MID(…,TRUE,5)`); a start or length around 1E+15 is `#VALUE!`
(`MID("ü",1000000000000000.0,4)`); a length a hair below 1 (0.999999999999999) counts as 1.
Evidence: gen-m2/ef5c55a81783b7d5 A40, d0c29dd5c530e28c A269, 78bc88199fc96650 A345,
b4392a6514a36ade A14 (Calc sheet). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0219

## Text comparison uses Excel's collation, not code-point order
Rule: `<`, `>`, `>=` between texts follow Excel's collation: "x" >= "ü" is TRUE, "1.5" > "?b" is TRUE
(code-point order says otherwise for the first).
Evidence: gen-m2/2cfe279fe77a000d Calc!A318, 96a0952412cefa6c Calc!A284. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED (two cells; the full collation table needs probes, T-0220)
Tasks: T-0220
