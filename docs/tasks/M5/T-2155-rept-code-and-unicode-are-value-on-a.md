---
id: T-2155
title: "REPT, CODE and UNICODE are #VALUE! on a text argument over 255 characters"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2199]
---

## Goal
REPT, CODE and UNICODE are `#VALUE!` when their text argument is longer than 255 characters, at the exact cut Excel uses.

## Context
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2199 (research), which fills `probes/t2199-text-length-and-combin-cuts`. Use that case; do not probe again. Its failing cells are this task's cells too.
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). Rule: docs/excel-semantics/text.md "REPT, CODE and UNICODE refuse a text argument over 255 characters" (added by this triage):
600 characters fail, 255 work (`REPT(REPT("x",255),3)`, 033e65b9 A55). Probe the cut first (`excel-probe`: 256, 257 and 300
characters, an astral character straddling 255 UTF-16 units) and check whether other text functions with a one-character or
count argument (CHAR? LEFT? CLEAN?) share it; gen-m5 shows LENB, PROPER, UPPER, EXACT, FIND and SEARCH do not. Code:
`fn_rept`, `fn_unicode` (functions/text/common.rs), `fn_code` (char_code.rs); reuse `reject_over_limit`-style helpers in
functions/text/common.rs rather than adding a new length check.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 6 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/4d099bea7898ebf1 Calc!A359 | `=REPT(REPT("ab",300),(SUM(Data!D6:D7,Data!D11,Data!B9)^Data!$B$4))` | "#VALUE!" | "abababababababababababababababababababababab..." |
| gen-m5/6a8bb7cd6d7769ad Calc!A191 | `=REPT(REPT("ab",300),0)` | "#VALUE!" | "" |
| gen-m5/8a387c26d182e73c Calc!A228 | `=REPT(REPT("ab",300),3)` | "#VALUE!" | "abababababababababababababababababababababab..." |
| gen-m5/901e91c09b347914 Calc!A45 | `=REPT(REPT("ab",300),5)` | "#VALUE!" | "abababababababababababababababababababababab..." |
| gen-m5/b237245054b4fedb Calc!A394 | `=UNICODE(REPT("ab",300))` | "#VALUE!" | 97.0 |
| gen-m5/ecd1b828aebad1e8 Calc!A1 | `=CODE(REPT("ab",300))` | "#VALUE!" | 97.0 |

## Acceptance criteria
- [ ] These 6 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] text.md entry updated with the cut and marked implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- REPT's 32,767-unit result limit (already implemented).
