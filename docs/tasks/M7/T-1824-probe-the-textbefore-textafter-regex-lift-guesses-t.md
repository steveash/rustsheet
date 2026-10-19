---
id: T-1824
title: "Probe the TEXTBEFORE/TEXTAFTER/REGEX lift guesses T-1272 pins in tests"
milestone: M7
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1272]
---

## Goal
Excel's behavior is probed and recorded for the lift details T-1272 implemented as guesses, and our tests match the evidence.

## Context
T-1272's fidelity review: its tests pin behavior with no oracle cell. docs/excel-semantics/text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar arguments" lists these as UNVERIFIED: when two delimiters of a list match at the same position, the first in list order wins; every list entry longer than the text gives the single-delimiter error; a per-element REGEX compile error; if_not_found, pad_with, match_end and return_all lifting (if_not_found is now evaluated eagerly); REGEXEXTRACT return_all keeping the first value of a lifted element; and (first, as it is a common pattern) a 1x1 array lifting the call: `COLUMNS(TEXTSPLIT({"a,b"},","))`, `COLUMNS(TEXTSPLIT(FILTER(A1:A2,A1:A2="a,b"),","))`, `COLUMNS(REGEXEXTRACT({"a1b2"},"[0-9]",1))` are ours 1 since T-1272 (2 before). Tests: test/text_functions/fn_textbefore.rs, fn_textsplit.rs. Use the `excel-probe` skill. (An error and a boolean inside the delimiter array were settled by T-1361: the error, and the spelling. Left from T-1361's review: a scalar boolean TEXTSPLIT delimiter, `TEXTSPLIT("a TRUE b",TRUE)`, ours `#VALUE!` though a boolean list element is its spelling; which of two different errors in one delimiter list wins, ours the first in row-major order; and a delimiter-list error against an invalid instance_num, `TEXTBEFORE("abc",{"",#N/A},0)`, ours `#N/A`.)

## Acceptance criteria
- [ ] A probe case with oracle values covers each guess above; text.md marks each VERIFIED or records Excel's actual rule; any mismatch is fixed or filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/textsplit-lift-guesses (87 formulas, excel-graph). Mismatches fixed: the longest delimiter wins where several match at one position (TEXTSPLIT, TEXTBEFORE/AFTER; the guess was first in list order), a negative instance_num now checks a match at the start of the text, TEXTSPLIT's pad_with is `#VALUE!` as an array (it was lifted), a scalar boolean TEXTSPLIT delimiter is its spelling. Recorded in text.md; REGEXEXTRACT return_mode 2 and its case_sensitivity argument (cells A53, A55) stay failing and are T-1360's. Eval: probes case 88/90 cells.
