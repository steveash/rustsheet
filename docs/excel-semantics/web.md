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
Whether an array argument lifts is unprobed; ours gives one scalar `#VALUE!`.
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
