# Text

## Broadcast text output cap
Rule: a text function broadcast over arrays (LEFT, REPT, SUBSTITUTE, ...) returns a single scalar `#VALUE!` (no spill) once the total bytes of its string results exceed 64 MiB, to bound allocation on untrusted input (CLAUDE.md directive 5). Excel's behavior (expected, not measured): compute the whole array, limited only by memory and the 32,767-char per-cell limit. Ours differs on legitimate data too: the budget counts UTF-8 bytes (not Excel's UTF-16 characters, so CJK text hits it about 3x sooner in characters), and a full-column text formula such as `=UPPER(A:A)` over ~65-char strings exceeds 64 MiB and returns `#VALUE!` where Excel spills normally.
Evidence: none for Excel's behavior; this is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: functions/text/util.rs (`MAX_BROADCAST_TEXT_BYTES`, `within_text_budget`), checked by functions/text/common.rs (`broadcast_text`)

## SUBSTITUTE result longer than 32,767 characters
Rule: a SUBSTITUTE result over 32,767 characters is `#VALUE!` (unlike CONCATENATE and `&`, which truncate), for
replace-all and for a given instance number alike; exactly 32,767 is fine, and a non-matching instance leaves
the text unchanged. `CONCAT` and `TEXTJOIN` past the limit give `#CALC!` in excel-graph (not implemented).
Evidence: probes/text-result-limit C1-C7 (SUBSTITUTE), C8-C9 (CONCAT/TEXTJOIN `#CALC!`) (excel-graph, 2026-10-05;
excel-desktop not used).
Status: VERIFIED for SUBSTITUTE; CONCAT/TEXTJOIN `#CALC!` unverified on desktop
Implemented in: `substitute_element` in functions/text/common.rs sizes the result before building it (T-0241)
Tasks: T-0241
