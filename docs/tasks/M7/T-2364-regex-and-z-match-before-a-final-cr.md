---
id: T-2364
title: "REGEX `$` and `\\Z` match before a final CR or CRLF, as before a final LF (t2364-regex-dollar-final-cr A1-A4)"
milestone: M7
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
In the REGEX* functions, `$` without `(?m)` (and `\Z`) also match before a final CR and before a final CRLF, as
they already do before a final LF. This is what Excel does.

## Context
- Found by T-2287's probe and moved to probes/t2364-regex-dollar-final-cr (excel-graph, 2026-10-22). These are its 4 known failures:

  | cell | formula | Excel | ours |
  |---|---|---|---|
  | A3 | `REGEXTEST("a"&CHAR(13),"a$")` | TRUE | FALSE |
  | A4 | `REGEXTEST("a"&CHAR(13)&CHAR(10),"a$")` | TRUE | FALSE |
  | A1 | `LEN(REGEXEXTRACT("ab"&CHAR(13)&CHAR(10),"\X$"))` | 1 ("b") | 2 (CRLF) |
  | A2 | `CODE(REGEXEXTRACT("ab"&CHAR(13)&CHAR(10),"\X$"))` | 98 | 13 |

  A5 (`"a\r$"` on `"a"&CHAR(13)&CHAR(10)`, TRUE) shows `$` still matches before the final LF of a CRLF. Together the
  cells read as PCRE2's ANYCRLF (or ANY) newline convention. docs/excel-semantics/text.md "REGEX* character classes are
  Unicode; `$` and `.` around line breaks" says a CR or U+2028 *inside* the text does not end a line. Only the final
  newline is probed.
- Code: `Anchor::EndOrFinalLf` in functions/text/pcre/backtrack.rs `holds`, the automata translation in pcre/hir.rs
  (`FinalLf`), and `final_lf_for` in pcre/mod.rs.
- Before implementing, probe: `$` before a final U+2028 / U+0085 / VT (to tell ANY from ANYCRLF); `(?m)$` and `(?m)^`
  around an inner CR and an inner CRLF; whether `.` matches CR; `\Z` before a final CR.

## Acceptance criteria
- [x] probes/t2364-regex-dollar-final-cr A1-A4 pass, and their lines leave eval/baseline/probes.txt with no
      new known failure.
- [x] A probe case (excel-graph) answers the questions above, and text.md records the newline rule (VERIFIED).
- [x] `make check` green; reviewers approve.

## Out of scope
- `$` with another LF earlier in the text (T-2355).

## Outcome
Probed the open questions first (probes/t2364-regex-newline-convention, 25 cells, excel-graph 2026-10-25): Excel's
PCRE2 uses the ANYCRLF newline convention. CR, LF and CRLF are newlines; U+2028, NEL, VT and FF are not. `$` and `\Z` match
before a final LF, CR or CRLF, and between the CR and LF of a final CRLF. `(?m)$` matches before every CR and LF. `(?m)^`
matches after an LF or after a CR that no LF follows. `.` matches neither CR nor LF. The engine now follows this.
The backtracker's `holds` and `.` read the rules directly. The automata use regex-syntax's CRLF forms
(`Dot::AnyCharExceptCRLF`, `Look::StartCRLF`, `Look::EndCRLF`). A new text kind, `FinalLf::Cr`, sends a regular pattern
holding `$`, `\Z` or `(?m)$` to the memoized mode on a text with a CR. text.md has a new VERIFIED entry, and SPEC-0013
section 2 notes the fourth text kind. Eval: probes known failures 2570 -> 2566 (A1-A4 fixed), plus 25 new passing cells,
no regressions. Unprobed: `(?m)^` at the very end after a final newline (unchanged).
