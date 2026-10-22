---
id: T-2287
title: "REGEX functions match \R and \X as PCRE2 does (regex-ascii-classes A27, A42)"
milestone: M7
lane: functions-text
status: done
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-1900]
---

## Goal
`\R` (any line break) and `\X` (an extended grapheme cluster) are accepted and match in REGEXTEST, REGEXEXTRACT and
REGEXREPLACE as Excel's PCRE2 matches them, instead of `#VALUE!`.

## Context
Gap triage 2026-10-21 (visibly-off cells, decision 0083). This is the follow-up SPEC-0013 and T-1973's acceptance
criteria name ("`\R` and `\X` parse and stay `#VALUE!`; one follow-up task, depends on T-1900, implements them"),
filed now so the two visibly-off cells have an owner. docs/excel-semantics/text.md "REGEX* character classes are
Unicode; `$` and `.` around line breaks (T-1829)" records that Excel accepts both.

Failing cells (probes/regex-ascii-classes, excel-graph, 2026-10-19; known failures in eval/baseline/probes.txt):

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/regex-ascii-classes Sheet1!A27 | `=IFERROR(REGEXTEST("é","\X"),"E"&ERROR.TYPE(REGEXTEST("é","\X")))` | TRUE | "E3" (`#VALUE!`) |
| probes/regex-ascii-classes Sheet1!A42 | `=IFERROR(REGEXTEST(CHAR(10),"\R"),"E"&ERROR.TYPE(REGEXTEST(CHAR(10),"\R")))` | TRUE | "E3" (`#VALUE!`) |

Root cause: the pattern front end rejects `\R` and `\X` (T-1973 makes them parse but still `#VALUE!`; both need the
backtracker of T-1900, since `\R` is atomic `(?>\r\n|\n|\x0b|\f|\r|\x85|\x{2028}|\x{2029})` in PCRE2 and `\X` is a
grapheme-cluster walk). The two cells only show that Excel accepts them on a one-character text; what a match spans
(`\r\n` as one `\R`, `e` plus a combining acute as one `\X`, an emoji ZWJ sequence) is unprobed. Probe that first
(excel-probe skill: REGEXEXTRACT all-matches mode and LEN of each match over `CHAR(13)&CHAR(10)`, `CHAR(11)`,
`CHAR(12)`, `UNICHAR(133)`, `UNICHAR(8232)`, `"e"&UNICHAR(769)`, a flag emoji and a ZWJ family emoji), record it in
text.md, then implement on T-1900's `pcre::Program` (no second matcher; reuse any grapheme segmentation the engine
already has, search with `rg grapheme` before adding a dependency).

## Acceptance criteria
- [x] probes/regex-ascii-classes A27 and A42 pass, and these 2 known-failure lines disappear from
      eval/baseline/probes.txt (`make eval-baseline`) with no new ones in any baseline.
- [x] A new probe case (excel-graph values) pins the match spans above; its cells pass, and text.md records the rule
      (VERIFIED) with an "Ours" line.
- [x] Directive 5: `\X` and `\R` charge the call's work counter per character like the other backtracker steps; a
      unit test runs `\X+` over a 32,767-character text within the call budget.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other PCRE2 escapes (`\N`, `\v`, scripts): T-1972 probes them.

## Visible-parity stage
Decision 0083: A27 and A42 are wrong-type cells (an error where Excel gives TRUE), so this task is in the campaign.
Its target is that every cell it owns has Excel's type, error and value; the acceptance criteria above are that
target.

## Outcome
- Probe probes/t2287-regex-r-x-spans (83 cells, excel-graph, 2026-10-22) pins what `\R` and `\X` match. Both are
  atomic. `\X` is UAX #29 without GB9c, plus two quirks of PCRE2's join table: an indicator joins only an indicator, and
  a Prepend does not join an emoji. Recorded in text.md "`\R` and `\X` are atomic; `\X` is PCRE2's grapheme cluster".
- `\R` is parsed into PCRE2's atomic group. `\X` is a new backtracker instruction (`Inst::Grapheme`) built on
  `pcre/grapheme.rs`, which reads its break values from regex-syntax's Unicode tables. No new dependency and no second
  matcher. `Ast::Unimplemented` is gone.
- Directive 5: each character `\X` takes is charged one step. Unit tests cover `\X+` over 32,767 characters within the
  budget, and one 32,767-character cluster searched from every start.
- Eval delta: probes/regex-ascii-classes A27 and A42 fixed; the new case passes 83/83. The probe also found that `$`
  matches before a final CR or CRLF. Those 5 cells moved to probes/t2364-regex-dollar-final-cr, and its 4 known failures
  are owned by the follow-up T-2364. There are no other new known failures.
- Follow-up: T-2364.
