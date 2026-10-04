# Criteria (COUNTIF/SUMIF family, D-functions, wildcard lookups)

## "<>text" counts non-text cells
Rule: `COUNTIF(range,"<>x")` counts every cell not equal to `x`: numbers, booleans, errors and blanks included.
Evidence: smoke/criteria Sheet1!C6 (hand-computed, well-known Excel behavior). probes/criteria-grammar
has no `"<>text"` cell, so this is still unprobed; T-0011 adds one. (Its C5 `"<>"` = 6 counts
every non-blank cell, the empty-string cell included.)
Status: UNVERIFIED (probe pending)
Implemented in: functions::criteria::result_is_not_equal_to_string

## Wildcard text matching
Rule: `*` and `?` match whole text values; non-text never matches; the pattern `"abc~"` does not
match "abc".
Evidence: probes/criteria-grammar C8 (`"abc~"` → 0), C9 (`"*"` → 4: the four text cells including
"" and the text "#N/A"; the number and TRUE excluded), C10 (`"a?c"` → 1). Oracle: excel-graph, 2026-10-04.
Status: VERIFIED

## Wildcard escapes, case and trailing tilde
Rule (hypothesis): `~*`, `~?`, `~~` escape the wildcard characters, matching is case-insensitive,
and a trailing lone `~` is a literal tilde (so `"abc~"` would match the text "abc~").
Evidence: smoke/criteria Sheet1!C1,C3,C4,C10,C11 (hand). Probe cells: T-0011.
Status: UNVERIFIED (probe pending)
Implemented in: functions::criteria::wildcard_text_matcher

## D-function plain-text equality matches by prefix
Rule: in DSUM/DCOUNT/DGET/... a plain text criterion (no operator, no wildcard) matches cells that *start with* it ("ab" matches "abc"); every other part of the grammar is shared with COUNTIF.
Evidence: probes/criteria-grammar C11 (`DCOUNTA` with criterion "ab" over ab, abc, xab → 2).
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Implemented in: functions::criteria::build_criteria_with (CriteriaFlavor::Database)

## D-function criteria text is not trimmed
Rule: DSUM/DCOUNT/... use criteria text exactly as written; with surrounding spaces, `"  > 1 "`
is not the operator `>` (if Excel stripped the leading spaces, the operand `" 1 "` would parse as
1 and match), so it matches nothing (DSUM over Qty 1, 2, 4 → 0, not 6). Disproves the earlier
"criteria text is trimmed first" hypothesis, which the engine had implemented.
Evidence: smoke/d-function-criteria Sheet1!J2 (`=DSUM(A1:B4,"Qty",F1:F2)`, F2 = `"  > 1 "` → 0).
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED (only the both-sides-padded case; leading-only / trailing-only spaces: T-0011)
Implemented in: functions::database::criteria_cell_matches (no trimming)

## "=" does not match an empty-string cell
Rule: `COUNTIF(range,"=")` does not count a cell holding the empty string, while
`COUNTIF(range,"")` does. (That `"="` counts truly blank cells is not covered yet: the probe range has none; T-0011.)
Evidence: probes/criteria-grammar C3 (`""` → 1), C4 (`"="` → 0) over a range with one empty-string
cell and no blank cells. Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
