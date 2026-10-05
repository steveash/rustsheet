# Coercion and argument handling

## AND / OR evaluate every argument; an error anywhere wins
Rule: AND/OR do not short-circuit: `=OR(TRUE,(2<COUNTIFS(…shape mismatch…)))` → `#VALUE!`.
Evidence: gen-m2/78bc88199fc96650 Calc!A217; OR: 1b6ceed7489e0800 A143, 893302d6c5b99e2e A170, dd99be1f2454159f A3; AND: 75840b9c71a50596 A56, 96a0952412cefa6c A126. Unprobed: which error wins with several errors, and an error inside the same range as the deciding value (T-0229). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0215

## MIN / MAX coerce value arguments like SUM; ranges and arrays give only their numbers
Rule: a value argument (literal or function result, e.g. IFERROR or CONCATENATE) is coerced like
SUM's: non-numeric text is `#VALUE!` (`=MIN("a")`, `=MAX(CONCATENATE("x"))`), numeric text counts
(`=MIN("2")` → 2, `=MAX("1e2",5)` → 100), TRUE/FALSE count as 1/0 (`=MIN(TRUE,5)` → 1), and an empty
argument counts as 0 (`=MIN(5,)` → 0, `=MAX(-5,)` → 0). A reference, also one returned by IF/INDEX
(references.md), and an array contribute only their numbers: `=MIN(A1)` with A1 = "a" → 0,
`=MAX(IF(TRUE,A1),A2)` → A2, `=MIN({TRUE,5})` → 5, `=MAX({"9",-1})` → -1. IFERROR returns a value,
so `=MIN(IFERROR(A1,1))` with A1 = "a" is `#VALUE!` and an IFERROR of a blank cell is 0.
Evidence: probes/min-max-value-args (C1–C16); gen-m2/f7e6d351c8538ef5 Calc!A14,
dd99be1f2454159f A73/A264, 2cfe279fe77a000d A138, 04cfd82f54dbc2b2 A182.
Oracle: excel-graph, 2026-10-05 (probe), 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Implemented in: functions/math_and_trigonometry/mathematical.rs (`fold_min_max`)
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
The same holds for a one-cell intersection of a whole row: `('Q''1'!5:5 'Q''1'!B5)+2` with B5 = " 2 "
is 4 (gen-m1/7cb99acdd82402f1 Calc!A72, A103, A217, A235; found in T-0105).
Status: VERIFIED
Tasks: T-0131

## An error in a range operand of arithmetic wins over the right operand's error
Rule: when the left operand is a one-cell range holding an error and the right operand is an error,
the result is the left error: `(Data!C1:C10 Data!B9:D9)/Data!D$5` with C9 = `#N/A` and
D5 = `#DIV/0!` is `#N/A`. RustSheet gives `#DIV/0!`, because the range becomes an array whose
elements are only looked at after the right operand's error has already been returned.
Evidence: gen-m1/4dc8ed1de05a61be Calc!A250 (oracle: excel-graph, 2026-10-05; found in T-0105).
Status: VERIFIED (one cell; the multi-cell case is unprobed)
Tasks: T-0137
