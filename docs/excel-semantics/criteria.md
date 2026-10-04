# Criteria (COUNTIF/SUMIF family, D-functions, wildcard lookups)

## "<>text" counts non-text cells
Rule: `COUNTIF(range,"<>x")` counts every cell not equal to `x`: numbers, booleans, errors and blanks included.
Evidence: smoke/criteria Sheet1!C6 (hand-computed, well-known Excel behavior). probes/criteria-grammar
has no `"<>text"` cell, so this is still unprobed (its C5 `"<>"` = 6 counts every non-blank cell,
the empty-string cell included).
Status: UNVERIFIED (probe pending: needs a `"<>text"` cell)
Implemented in: functions::criteria::result_is_not_equal_to_string

## Wildcard text matching
Rule: `*`, `?` and `~` escapes (`~*`, `~?`, `~~`) match whole text values, case-insensitively; non-text never matches.
A trailing lone `~` is a literal tilde (`"abc~"` does not match "abc").
Evidence: smoke/criteria Sheet1!C1,C3,C4,C10,C11 (hand); probes/criteria-grammar C8 (`"abc~"` → 0),
C9 (`"*"` → 4: the four text cells including "" and the text "#N/A"; the number and TRUE excluded),
C10 (`"a?c"` → 1). `~*`/`~?` escapes and case-insensitivity are not covered by the probe.
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED (escapes and case not probed)
Implemented in: functions::criteria::wildcard_text_matcher

## D-function plain-text equality matches by prefix
Rule: in DSUM/DCOUNT/DGET/... a plain text criterion (no operator, no wildcard) matches cells that *start with* it ("ab" matches "abc"); every other part of the grammar is shared with COUNTIF. Criteria text is trimmed first.
Evidence: probes/criteria-grammar C11 (`DCOUNTA` with criterion "ab" over ab, abc, xab → 2).
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Implemented in: functions::criteria::build_criteria_with (CriteriaFlavor::Database)

## "=" does not match an empty-string cell
Rule: `COUNTIF(range,"=")` does not count a cell holding the empty string, while
`COUNTIF(range,"")` does. (That `"="` counts truly blank cells is not covered: the probe range has none.)
Evidence: probes/criteria-grammar C3 (`""` → 1), C4 (`"="` → 0) over a range with one empty-string
cell and no blank cells. Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
