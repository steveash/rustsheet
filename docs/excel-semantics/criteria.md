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
An explicit leading `=` (`"=ab"`) turns the prefix match into an exact match (probes/criteria-operands D18).
Evidence: probes/criteria-grammar C11 (`DCOUNTA` with criterion "ab" over ab, abc, xab → 2).
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Implemented in: functions::criteria::build_criteria_with (CriteriaFlavor::Database)

## D-function criteria are not trimmed into an operator criterion
Rule: a D-function criterion padded on both sides, `"  > 1 "`, does not act as `>1`: DSUM over
Qty 1, 2, 4 returns 0, not 6. This disproves the earlier "criteria text is trimmed first"
hypothesis (which the engine had implemented). The evidence fits two explanations, still to be
told apart: no trimming at all (the text is a plain-text criterion), or leading spaces trimmed
but the operand `" 1 "` kept as text (matching no number). The engine implements "no trimming".
Evidence: smoke/d-function-criteria Sheet1!J2 (`=DSUM(A1:B4,"Qty",F1:F2)`, F2 = `"  > 1 "` → 0).
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED for the both-sides-padded case; which explanation holds (and leading-only /
trailing-only padding, `">"&" 1 "`) is UNVERIFIED (probe pending, T-0011)
Implemented in: functions::database::criteria_cell_matches (no trimming)

## "=" does not match an empty-string cell
Rule: `COUNTIF(range,"=")` does not count a cell holding the empty string, while
`COUNTIF(range,"")` does. (That `"="` counts truly blank cells is not covered yet: the probe range has none; T-0011.)
Evidence: probes/criteria-grammar C3 (`""` → 1), C4 (`"="` → 0) over a range with one empty-string
cell and no blank cells. Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Tasks: T-0227

## `<`, `>`, `<=`, `>=` with a boolean or error operand
Rule: `COUNTIF(r,">TRUE")`, `"<TRUE"`, `">#N/A"` and `"<#N/A"` match nothing, even over TRUE or
#N/A cells; `"<=TRUE"`/`">=TRUE"` (and `<=`/`>=` with `#N/A`) behave as equality and count exactly
the matching boolean / error cells. The operand is never compared as text ("abc" is not <= TRUE).
Evidence: probes/criteria-operands C1-C8 over 10, "abc", TRUE, #N/A, "abd", 5: 0,0,1,1,0,0,1,1.
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Implemented in: functions::criteria::bool_or_error_operand_matcher

## Criteria kinds in COUNTIF and D-functions (more evidence)
Rule: `"<>abc"`, `"<>TRUE"`, `"<>#N/A"` count every non-matching cell (6 of 7, blank included);
`"=TRUE"` / `"=#N/A"` count the boolean / error cells; `">abc"`/`"<abd"` compare text. In
D-functions (DCOUNTA) `"<>ab"`, `"a*"`, `"ab?"`, a numeric criterion cell, a boolean criterion
cell and `"<>3"` follow the shared grammar; an explicit `"=ab"` is exact, not a prefix (1 over ab, abc, xab, 3, TRUE, where plain `ab` gives
2), while an error criterion cell (`=NA()`) matches nothing.
Evidence: probes/criteria-operands C9-C16, D17-D25. Oracle: excel-graph, 2026-10-04.
Status: VERIFIED

## COUNTIFS lifts an array criterion
Rule: an array constant as a criterion makes COUNTIFS return an array of counts, one per element:
`SUM(COUNTIFS(A1:F1,{"bar","baz"}))` = 3 and `{"foo","bar"}` = 4.
Evidence: poi/70005-countifs.xlsx Sheet1!A3, D3 (3), A4, D4 (4) (oracle: excel-graph, 2026-10-04).
Status: VERIFIED
Tasks: T-0207

## SUMIFS / COUNTIFS: every range must have the first range's shape
Rule: if any criteria (or sum) range is shaped differently from the first range, SUMIFS/COUNTIFS
return `#VALUE!`, even when the cell counts are equal (2x4 vs 4x2). `=SUMIFS(Data!B7:C9,Data!A10:D12,"<5")`
→ `#VALUE!`; all 145 mismatched top-level calls in gen-m2 are `#VALUE!`, all 26 same-shape calls are numbers.
Evidence: gen-m2/04cfd82f54dbc2b2 Calc!A150, 7469c3e920e1ade4 Calc!A51. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0209

## SUMIF resizes sum_range from its top-left cell and never errors
Rule: SUMIF sums the range of the criteria range's shape that starts at sum_range's top-left cell,
whatever sum_range's own shape: `=SUMIF(Data!C2:E2,"<>1",Data!B4:B10)` sums B4:D4 (123456831.123).
71 of 71 mismatched calls in gen-m2 return numbers.
Evidence: gen-m2/629e6e8b76fab8b9 Calc!A217. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0214
