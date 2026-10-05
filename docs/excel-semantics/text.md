# Text

## Broadcast text output cap
Rule: a text function broadcast over arrays (LEFT, REPT, SUBSTITUTE, ...) returns `#VALUE!` once the total bytes of its string results exceed 64 MiB, to bound allocation on untrusted input (CLAUDE.md directive 5). Excel's real behavior for such inputs is not measured.
Evidence: none for Excel's behavior; this is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: functions/text/util.rs (`MAX_BROADCAST_TEXT_BYTES`, `within_text_budget`), checked by functions/text/common.rs (`broadcast_text`)
