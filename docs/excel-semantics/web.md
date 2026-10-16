# Web functions (ENCODEURL, FILTERXML, WEBSERVICE)

WEBSERVICE is out of scope (decision 0027; eval/catalog/functions.yaml).

## FILTERXML is `#VALUE!` in Excel for the web
Rule: in Excel for the web, `FILTERXML("<a>1</a>","//a")` (well-formed XML, valid XPath) is
`#VALUE!`. Microsoft's FILTERXML page says the function relies on Windows and does not return
results on Mac; what desktop Excel on Windows returns is not yet measured.
Evidence: probes/catalog/function-name-forms A7 (ERROR.TYPE 3 = #VALUE!); probes/catalog/availability-14
A7 (`FILTERXML($Z$1,$Z$1)` is recognised, not #NAME?) (oracle: excel-graph, 2026-10-07, T-0040).
Quirk: an error in the first argument propagates instead of the `#VALUE!` (gen-m6/abcfd9979f316267 A9
`FILTERXML(D4&NA(),..)` is `#N/A`; gen-m7/4629e931955aa4b8 A34 gives DROP's `#CALC!`).
An error only in the second argument propagates too
(`FILTERXML("x",NA())` is `#N/A`), and with errors in both the first argument wins, left to right
(`FILTERXML(1/0,NA())` is `#DIV/0!`, `FILTERXML(NA(),1/0)` is `#N/A`).
Evidence: probes/filterxml-error-arguments A1:A6 (oracle: excel-graph, 2026-10-15, T-1570).
An array argument lifts: `FILTERXML({"<a>1</a>";"<a>2</a>"},"//a")` spills two `#VALUE!` (probes/filterxml-evidence-floor D27:D28, T-1704). Probed too (probes/filterxml-array-forms, excel-graph, T-1722): a range as the first argument (`FILTERXML(Z1:Z2,"//a")`) and an array as the second (`FILTERXML("<a/>",{"//a","//b"})`) give `#VALUE!` per element; an error element passes through while the others stay `#VALUE!` (`FILTERXML({"x";#N/A},"//a")` is `#VALUE!`,`#N/A`); a scalar error next to an array broadcasts (`FILTERXML(NA(),{"//a";"//b"})` spills two `#N/A`, `FILTERXML({"<a/>";"<b/>"},NA())` too); a lambda first argument is `#VALUE!` even with `NA()` second (`FILTERXML(LAMBDA(x,x),NA())`).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)
Implemented in: crates/rustsheet-engine/src/functions/text/filterxml.rs (T-1255, decision 0039); no XML is parsed.

## ENCODEURL is `#VALUE!` in Excel for the web
Rule: in Excel for the web, ENCODEURL is `#VALUE!` for every argument probed: `"a b&c"`, `"abc"`, a text cell, `1`
and `TRUE`. It lifts over an array or a range, and each element is `#VALUE!` (`ENCODEURL({"a b","c"})` spills two). As with
FILTERXML, the function seems not to work on the web. The engine follows the golden (decision 0039 rule 4) and encodes nothing; Microsoft's page
describes percent-encoding (`"a%20b%26c"`), which desktop Excel may do. An error argument propagates as it came (unprobed).
Evidence: probes/info-logic-argument-modes-2 A1:A6, B5; probes/info-logic-argument-modes A15:B15 (oracle: excel-graph, 2026-10-09, T-1088).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081). Implemented in: crates/rustsheet-engine/src/functions/text/encodeurl.rs (T-1219).
Tasks: T-1088, T-1219

## Evidence floor probes (T-1628)
Rule: on excel-graph every ENCODEURL cell probed is `#VALUE!` whatever the text (reserved characters, non-ASCII,
a surrogate pair, `""`, numbers, booleans, a date, a cell reference, a range and an array literal, which spill as
`#VALUE!`), including `ENCODEURL(ENCODEURL("a b"))` and `LEN(ENCODEURL("a b"))`; an error argument propagates
(`ERROR.TYPE(ENCODEURL(1/0))` is 2). Every FILTERXML cell probed is `#VALUE!` too: well-formed and malformed XML,
no-match XPath, several matches, attributes, `count()`/`string()`, numeric text, a non-text argument and an array argument.
Evidence: probes/encodeurl-evidence-floor, probes/filterxml-evidence-floor (oracle: excel-graph, 2026-10-16, T-1628).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081). The web results say nothing about desktop
percent-encoding or XPath behavior.

## GETPIVOTDATA without a pivot table (T-1628)
Rule: with no pivot table in the workbook, `GETPIVOTDATA("Sales",ref,...)` is `#REF!` for every reference probed
(a cell outside any pivot, a range, `OFFSET`/`INDIRECT` results, extra field/item pairs, an odd number of
pairs, `""` as data_field, a numeric pivot_table or numeric item). An error in either of the first two arguments
propagates as it came (`pivot_table` `NA()` is `#N/A`, `1/0` is `#DIV/0!`; `data_field` `NA()` is `#N/A`).
`ERROR.TYPE(GETPIVOTDATA("Sales",A1))` is 4. Which error wins when both are errors, a boolean data_field and a
reference to an existing second sheet are unprobed.
Evidence: probes/getpivotdata-evidence-floor (oracle: excel-graph, 2026-10-16, T-1628).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081).
A real pivot: see the next section (T-1705).

## GETPIVOTDATA against a pivot table (T-1705)
Rule: against a compact-form pivot (one row field, an optional column field, one data field), GETPIVOTDATA
returns the value of the pivot cell the field/item pairs pick, including subtotal and grand-total cells:
- `data_field` is the source field name (`"Sales"`) or the data field's caption (`"Sum of Sales"`), matched
  without regard to case (`"sales"`). Padding is not trimmed: `" Sales "` is `#REF!`. A caption of another
  aggregate (`"Sum of Sales"` on an Average pivot) and an unknown field (`"Profit"`) are `#REF!`. A blank cell
  as `data_field` gives the grand total (one data field; `""` with no pivot is `#REF!`, see above).
- No pairs give the grand total, one pair a row or column total, two pairs their intersection in either
  order. Field and item names ignore case (`"region","east"`; `"Product","a","Region","EAST"`). An item may
  be a reference to a label cell (`A5`).
- `#REF!`: an item not in the field (`"North"`, `" East"`), a field not in the pivot (`"Market"`, or a
  source field the pivot does not show), the same field twice with different items, and a field with no
  item (an odd argument count).
- An intersection that has no source rows (shown blank) is **0**, not `#REF!` (West/C).
- `pivot_table` may be any cell of the pivot (`$C$6`, inside the data area) or a range covering it
  (`$A$3:$E$7`); a cell outside every pivot (`$H$30`, or the source data `Data!$A$1`) is `#REF!`.
- An array of items lifts: `"Region",{"East";"West"}` spills 22, 27 and `"Product",{"A","B","C"}` spills
  34, 12, 3.
Evidence: probes/getpivotdata-real-pivot (oracle: excel-graph, 2026-10-16, T-1705). The pivots are written
by the case builder (`pivot_tables`, tools/src/rustsheet_tools/pivot.py); excel-graph rendered them cell for
cell as the builder does (a one-off check of the label, value, blank and grand-total cells, not committed).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081). Not probed: several data fields (the
`Values` field), two row fields and their subtotals, filters, hidden items, dates and numbers as items, and
pivots in tabular or outline form.
