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
Implemented in: `Model::array_node_to_number` (crates/rustsheet-engine/src/cast.rs), via `cast_number`; only the " 2 " form is oracle-covered, other text forms follow the scalar path.
Tasks: T-0131

## An error in a range operand of arithmetic wins over the right operand's error
Rule: when the left operand is a one-cell range holding an error and the right operand is an error,
the result is the left error: `(Data!C1:C10 Data!B9:D9)/Data!D$5` with C9 = `#N/A` and
D5 = `#DIV/0!` is `#N/A`. RustSheet gives `#DIV/0!`, because the range becomes an array whose
elements are only looked at after the right operand's error has already been returned.
Evidence: gen-m1/4dc8ed1de05a61be Calc!A250 (oracle: excel-graph, 2026-10-05; found in T-0105).
Status: VERIFIED (one cell; the multi-cell case is unprobed)
Tasks: T-0137

## SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing
Rule: the two arrays are paired element by element in row-major order and need only the same
element count, not the same shape (a 3x2 range pairs with a 6x1 range); a pair where either side
is not a number (blank, text, boolean in a range) is skipped; no numeric pair left is `#DIV/0!`;
different element counts are `#N/A`; scalar arguments are allowed (`SUMXMY2(2,4)` = 4).
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!D1336, E1336,
G1336, I1336, J1336 (`#N/A`), K1336, E1340, F1340, H1340, J1340 (`SUMX2PY2(G7:H9,J7:J12)` =
105.7931362501), K1340, F1344, G1344, H1344 (`#N/A`), I1344, J1344.
Status: VERIFIED (Excel-cached)
Tasks: T-0944

## STDEV / VAR / DEVSQ / AVEDEV / SUMSQ typed arguments; AVEDEV and DEVSQ of nothing is #NUM!
Rule: numeric text and booleans typed directly as arguments are counted (`STDEV("1","2")` =
0.7071…, `VAR(TRUE,FALSE,TRUE)` = 0.333…, `SUMSQ("2","3",4)` = 29, `AVEDEV(TRUE,FALSE)` = 0.5),
while text and booleans in ranges are ignored. With no number at all AVEDEV and DEVSQ return
`#NUM!`, whereas STDEV/VAR/VARP return `#DIV/0!` (inconsistent, but that is Excel).
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!G1288, K1288,
G1448, K1448, G1456, K1456, I392, L176, M1332, N1332; `#NUM!`: F176, G176, H392; `#DIV/0!`: F1288,
F1448, F1456, H1288.
Status: VERIFIED (Excel-cached)
Tasks: T-0945

## MODE: text arguments and blank single-cell references are #VALUE!
Rule: a text value argument, even numeric text (`MODE(B8,"1",2,3)`), makes MODE `#VALUE!`, and so
does a single-cell reference to a blank cell: `MODE(B9,2,D7)` with B9 = 2 and D7 blank is `#VALUE!`
although 2 appears twice. Blanks and text inside a multi-cell range are ignored
(`MODE(C16:E16,-1.1)` = -1.1 with D16 blank). The text wins over a later error argument:
`MODE("1",1,#DIV/0!,2,3)` is `#VALUE!`. Not isolated yet: a single-cell reference to text on its own.
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!G936, H936,
K936, M936 (`#VALUE!`); D936, F936, I936, J936, L936.
Status: VERIFIED (Excel-cached)
Tasks: T-0947

## MAXA / MINA: single-cell references are read like ranges
Rule: a single-cell reference argument to text counts as 0 and to a boolean as 1/0, as in a range:
`MAXA(F14,E19,F12)` over three text cells is 0, `MINA(B16,B10,B9)` with B16 = TRUE is 1.
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!J888, F912.
Status: VERIFIED (Excel-cached)
Tasks: T-0949
