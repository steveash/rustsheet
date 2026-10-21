---
id: T-2202
title: "Probe which format-code tokens make TEXT #VALUE! (T-2157)"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
excel-graph evidence that TEXT's code check applies to a text or boolean value for each class of invalid code, and of
which token makes `TEXT(2.5,"a*?b1")` `#VALUE!`. T-2157 can then write one validation rule from evidence, without probing.

## Context
- The M5 exit needs gen-m5 at 99.9% (decision 0085 item 5): at most 22 known failures; 190 are left today. T-2157
  (functions-text, opus) owns 11 and asks to "probe which token Excel rejects ... before changing the lexer". The
  functions lanes are full, so the probe moves here.
- Rule so far: docs/excel-semantics/number-formats.md "TEXT checks the code before it looks at a text or boolean value
  (T-2157)". A code that is `#VALUE!` for a number is `#VALUE!` for text and booleans too. The 11 gen-m5 cells cover
  "TRUE", "FALSE", "#N/A", "HELLO " and "x~*0.1Hello World". The invalid classes for a number are in "Bare letters and
  digits in a number format" in the same file: `n`/`N`, a bare `E` without a sign, a date token beside digit
  placeholders, and a point with a date token (probes/text-format-bare-literals, T-1262).
- `TEXT(2.5,"a*?b1")` is `#VALUE!` (gen-m5/d1b251fd6e9c4b4b Calc!A120; ours "a "). Candidates: the `*?` fill, `b`
  (a date token) beside the digit `1`, or the pair. T-2157 lists `"a*?"`, `"*?b1"`, `"ab1"` and `"b1"`.
- Case tag: number-format codes are `M10` (decisions 0057, 0058). The task stays M5 because it unblocks T-2157.

## Acceptance criteria
- [ ] New case `eval/suites/probes/cases/t2202-text-code-check-on-text.case.json` (`rustsheet-case/v1`, one `M10` tag). For
      each code below it has TEXT of the number 2.5, the text "a", the text "" and the boolean TRUE. The codes:
      - the four from T-2157: `"a*?"`, `"*?b1"`, `"ab1"`, `"b1"`, plus `"*?"`, `"b"`, `"a*?b"`, `"*xb1"`;
      - one code per invalid class above (`"n"`, `"0E0"`, `"0b"`, `"0.b"`), each also with an `@` text section added
        (`"n;n;n;@"`, ...);
      - controls that are valid for a number: `"0.0"`, `"@"`, `"0;-0;0;@"`, `"\"x\""`, `"General"`.
- [ ] Its excel-graph sidecar is filled (`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`), with no
      pending cell.
- [ ] number-formats.md's T-2157 entry states which token of `"a*?b1"` Excel rejects. It says, per invalid class, whether
      the check also applies to text, empty text and booleans, and whether an `@` section changes that. It cites the case.
- [ ] The case's failing cells are new-coverage F lines in eval/baseline/probes.txt (`make eval-baseline`), with no
      regression in any suite. T-2157's Context names the case as its evidence and as cells it must pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the formatter or `fn_text` (T-2157).
- Text values under valid 1-3 section codes (T-1130); number-like text values (T-2158).
