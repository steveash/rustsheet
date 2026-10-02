# Arrays

## Empty-array result: CSE array formula vs dynamic array
Rule: In a legacy (CSE, `t="array"`) formula an empty result (DROP/TAKE to nothing, `SEQUENCE(0,0)`,
FILTER with no match) is cached by Excel as `#VALUE!`; dynamic-array formulas give `#CALC!`.
Evidence: ironcalc-calc/LOOKUP_AND_REFERENCE/FILTER_ERRORS.xlsx Sheet1!E1, DROP_TAKE.xlsx C2/E2/F2/I2/L2,
SEQUENCE.xlsx C16/E18 (excel-cached, Macintosh Excel; all `t="array"` single-cell refs).
Status: UNVERIFIED for the dynamic case (probe pending); the CSE case is cached-evidence only.
Implemented in: not implemented; engine does not distinguish CSE from dynamic anchors (see T-0904).
