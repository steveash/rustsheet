# Text

## Broadcast text output cap
Rule: a text function broadcast over arrays (LEFT, REPT, SUBSTITUTE, ...) returns a single scalar `#VALUE!` (no spill) once the total bytes of its string results exceed 64 MiB, to bound allocation on untrusted input (CLAUDE.md directive 5). Excel's behavior (expected, not measured): compute the whole array, limited only by memory and the 32,767-char per-cell limit. Ours differs on legitimate data too: the budget counts UTF-8 bytes (not Excel's UTF-16 characters, so CJK text hits it about 3x sooner in characters), and a full-column text formula such as `=UPPER(A:A)` over ~65-char strings exceeds 64 MiB and returns `#VALUE!` where Excel spills normally.
Evidence: none for Excel's behavior; this is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: functions/text/util.rs (`MAX_BROADCAST_TEXT_BYTES`, `within_text_budget`), checked by functions/text/common.rs (`broadcast_text`)

## LEFT/RIGHT/MID count arguments coerce numeric text and booleans
Rule: num_chars (and MID's start_num) read numeric text as a number and a boolean as 1/0, then
truncate like any number: `LEFT("a string","6")` → "a stri", `RIGHT("a string","6")` → "string",
`LEFT("a string",TRUE)` → "a", `RIGHT("a string",TRUE)` → "g"; with E17 = `="2.99999"` (text),
`LEFT("non-numeric",E17)` → "no" and `RIGHT(" beginsWithSpace",E17)` → "ce". A boolean held in a
referenced cell counts the same way (`MID(…,B4,5)` with B4 = TRUE starts at 1).
Evidence: poi/FormulaEvalTestData_Copy.xlsx EverythingTests!E824, H824, K824, E1168, H1168, K1168
(desktop Excel cached values); gen-m2/ef5c55a81783b7d5 Calc!A40 (excel-graph, T-0203).
Status: VERIFIED
Tasks: T-0256, T-0219
