---
id: T-3069
title: "Structured references: this-row FORMULATEXT, stored padded T[ Col ] is #REF!"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
FORMULATEXT of a this-row structured reference inside a table, and loading a stored padded bare column, match Excel.

## Context
T-2800 evidence (docs/excel-semantics/syntax.md, "Whitespace beside a missing argument"): inside a table Excel prints
`T[[#This Row], [Col]]` as `[@Col]` and the padded forms as `[ @Col ]`; a stored `SUM(T[ Col ])`, `T[ Col]` or
`T[Col ]` is `#REF!` (FORMULATEXT `=SUM(#REF!)`) and a stored `T[ #Data ]` makes the workbook unloadable (outside
our scope; ignore). Failing cases: probes/t2800-this-row-whitespace (B3:E3, F2, F3) and probes/t2800-stored-ws-0; stored-ws-1 and -2 (one-sided padding) already pass.
`[@ Col]` (F2) must also parse: it is `#ERROR!` for us, 5 in Excel.
Note T-2801/T-2962 chose the stored `T[ Col ]` round-trip for our own export; the export must then not write that form.

## Acceptance criteria
- [x] The listed probe cases pass; baseline updated (B3:E3 moved to T-3241, see Outcome).
- [x] Our xlsx export no longer writes a padded bare column Excel reads as `#REF!`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The unloadable-workbook case `T[ #Data ]`.

## Outcome
`[@ Col]` parses (the run after `@` is padding) and an unqualified `@` reference prints its padding (`[ @Col ]`), which
reads back. A padded bare column `T[ Col ]` is read as a column only in text the engine printed itself (edits re-parse
it); typed it is a parse error and loaded it is `#REF!`, as in Excel, and the xlsx export writes `T[ [Col] ]`.
Printing a qualified `T[[#This Row], [Col]]` as `[@Col]` in its own table (B3:E3) was split out to T-3241: a parse-time
flag broke copy and fill outside the table (PR #2415 review). Eval: probes known failures 2575 -> 2571
(t2800-stored-ws-0 A6:B6, t2800-this-row-whitespace F2:F3), no regressions. Follow-ups: T-3240 (probe), T-3241.

## Blocked because
Resumed after the review-round limit (step 5c).

Three review rounds ran on PR #2415 (session 2026-10-30). Round 3: security, architecture and quality APPROVE;
Excel fidelity REQUEST_CHANGES with one MAJOR finding: with the padded-bare-column reading off (typed or loaded), the
lexer skipped its rewind, so `T[ Col ]` swallowed text up to the next `)`. Fixed after round 3 on this branch (rewind
restored; tests `SUM(T[ Col ],1)` loaded is `SUM(#REF!,1)`, typed is a parse error); eval-check clean, not yet
re-reviewed. Open MINOR findings for the re-review: qualified `T[@ Col]` / `T[ @Col ]` and one-sided `[@Col ]` now
parse with padding (unprobed; add to T-3240); a one-sided nested `T[ [Col]]` prints `T[ Col]`, which an edit's
re-parse no longer reads as the column (pre-existing on main; follow-up or keep the nested brackets when printing
one-sided padding); a defined-name body `T[ Col ]` still reads as the column (unprobed); the model test's final
assertion accepts two outcomes; untyped `set_user_input` still accepts `T[ Col ]` (engine-text path).
