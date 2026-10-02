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
