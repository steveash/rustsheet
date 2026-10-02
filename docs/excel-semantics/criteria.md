# Criteria (COUNTIF/SUMIF family, D-functions, wildcard lookups)

## "<>text" counts non-text cells
Rule: `COUNTIF(range,"<>x")` counts every cell not equal to `x`: numbers, booleans, errors and blanks included.
Evidence: smoke/criteria Sheet1!C6 (hand-computed, well-known Excel behavior); oracle confirmation pending via probes/criteria-grammar.
Status: UNVERIFIED (probe pending)
Implemented in: functions::criteria::result_is_not_equal_to_string

## Wildcard text matching
Rule: `*`, `?` and `~` escapes (`~*`, `~?`, `~~`) match whole text values, case-insensitively; non-text never matches.
Evidence: smoke/criteria Sheet1!C1,C3,C4,C10,C11 (hand).
Status: UNVERIFIED (probe pending)
Implemented in: functions::criteria::wildcard_text_matcher

## D-function plain-text equality matches by prefix
Rule: in DSUM/DCOUNT/DGET/... a plain text criterion (no operator, no wildcard) matches cells that *start with* it ("ab" matches "abc"); every other part of the grammar is shared with COUNTIF. Criteria text is trimmed first.
Evidence: carried over from IronCalc's D-function implementation; oracle confirmation pending via probes/criteria-grammar (C11).
Status: UNVERIFIED (probe pending)
Implemented in: functions::criteria::build_criteria_with (CriteriaFlavor::Database)
