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
(`MID("ü",1000000000000000.0,4)`); in LEFT a length a hair below 1 (`TRUE - 1E-15`) counts as 1
(seen for LEFT only, and the cell cannot tell whether the subtraction or LEFT rounds: probe in T-0219).
Evidence: gen-m2/ef5c55a81783b7d5 A40, d0c29dd5c530e28c A269, 78bc88199fc96650 A345,
b4392a6514a36ade A14 (Calc sheet). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0219

## Text comparison uses Excel's collation, not code-point order
Rule: `<`, `>`, `>=` between texts follow Excel's collation: "x" >= "ü" is TRUE, "1.5" > "?b" is TRUE
(code-point order says otherwise for both: "x" < "ü" and "1" < "?").
Evidence: gen-m2/2cfe279fe77a000d Calc!A318, 96a0952412cefa6c Calc!A284. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED (two cells; the full collation table needs probes, T-0220)
Tasks: T-0220

## Double unary minus coerces to a number
Rule: `--x` is not a no-op. Each unary minus coerces its operand to a number: `--FALSE` = 0,
`--("">1.5)` = 1, `--""` (a cell holding empty text) = `#VALUE!`, `--100` = 100.
Evidence: gen-m1/6462bf910ef9f28f Calc!A49 (`--Data!C4`, C4 = FALSE), A84 (`--Data!$A11`, A11 = ""),
A17; gen-m1/57fe331cb7611dbf Calc!A167 (oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED
Tasks: T-0132

## Text in a range operand of arithmetic is coerced
Rule: arithmetic on a range coerces a numeric text cell to a number, so `Data!D7:D7/1` with
D7 = " 2 " (spaces around the digit) is 2 and `Data!D3:D3/0` with the same text is `#DIV/0!`. Every
observed number-producing text cell is " 2 ". That this is the same routine as scalar text
coercion is an inference for T-0131's probe to confirm.
Evidence: gen-m1/1ce5b2467b3b19fd Calc!A14, A79, A99; gen-m1/9646257ac139cd1e Calc!A25, A81 (oracle:
excel-graph, 2026-10-05, T-0109). The scalar form `-" 2 "*2` = -4 (gen-m1/10f579127ffb6d9b A106).
Status: VERIFIED
Tasks: T-0131
