---
id: T-1816
title: "Probe the TEXTBEFORE/TEXTAFTER/REGEX lift guesses T-1272 pins in tests"
milestone: M7
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1272]
---

## Goal
Excel's behavior is probed and recorded for the lift details T-1272 implemented as guesses, and our tests match the evidence.

## Context
T-1272's fidelity review: its tests pin behavior with no oracle cell. docs/excel-semantics/text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar arguments" lists these as UNVERIFIED: when two delimiters of a list match at the same position, the first in list order wins; every list entry longer than the text gives the single-delimiter error; an error inside the delimiter array; a boolean inside a TEXTBEFORE/TEXTAFTER delimiter array (`TEXTBEFORE("a TRUE b",{TRUE,"x"})` is ours `#VALUE!`, while a boolean in a delimiter range raises no error); a per-element REGEX compile error; if_not_found, pad_with, match_end and return_all lifting (if_not_found is now evaluated eagerly); REGEXEXTRACT return_all keeping the first value of a lifted element. Tests: test/text_functions/fn_textbefore.rs, fn_textsplit.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with oracle values covers each guess above; text.md marks each VERIFIED or records Excel's actual rule; any mismatch is fixed or filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
