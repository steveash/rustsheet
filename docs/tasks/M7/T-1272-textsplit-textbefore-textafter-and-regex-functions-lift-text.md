---
id: T-1272
title: "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX functions lift text over a range and scalar arguments over an array"
milestone: M7
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TEXTSPLIT, TEXTBEFORE, TEXTAFTER, REGEXTEST, REGEXEXTRACT and REGEXREPLACE lift their text over a multi-cell range and their scalar arguments (instance_num, pattern) over an array, while the delimiter arguments stay lists, as in Excel.

## Context
T-1083's probe probes/m7-argument-modes (excel-graph, 2026-10-09; docs/excel-semantics/arrays.md "Dynamic-array and LAMBDA function argument modes"). Failing rows: 13 and 15 (text over a range is `#VALUE!`), 16 TEXTBEFORE instance_num {1;2}, 17 TEXTAFTER (text lifts, the delimiter array is a list: 3x1, first "b"), 18-22 REGEX* (text over a range, pattern over an array), 55 `REGEXTEST(A1,1)` with A1 = 1 is TRUE in Excel (both read as text), ours `#VALUE!`. Rows 14 (`TEXTSPLIT("a,b-c",{",";"-"})` 1x3) already pass: keep the delimiter list behaviour. Code: functions/text/. Reuse the existing lift helpers.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/59cc50701f4937c9 Calc!A24 `=TEXTAFTER((VSTACK(Data!D1:D3)&("ü"&" x "))," ",SECOND(Data!B3),FILTER(Data!B1:E12,(Data!C1:C12<42)))`: Excel `#SPILL!`, engine `#N/IMPL!` (text and match_mode arrays).
- gen-m7/93976dd9a3578ed9 Calc!A11 `=(YEAR(42463)-TEXTBEFORE(CHOOSEROWS(Data!A5:B12,1,1),(Data!A2&Data!E$2)))`: Excel `#SPILL!`, engine `#N/IMPL!`.
- gen-m7/9ff3afcacd3bd0fc Calc!A35 `=TEXTSPLIT((INDEX(Data!D7:E10,0,1)&CELL("contents",Data!$E7)),Data!A8:D11)`: Excel `#SPILL!`, engine `#N/IMPL!` (its 4x4 delimiter range also needs T-1361).
- gen-m7/6db767d720c5b55a Calc!A28 (TEXTBEFORE of a BYROW result, Excel `#SPILL!`) also needs T-1273 and T-1361.
Delimiter ranges (a multi-cell range with blanks, numbers or booleans as the delimiter list) are T-1361; REGEX optional arguments T-1360.

## Acceptance criteria
- [x] Probe rows 13, 15-22 and 55 and cells K11-K13 pass; baseline updated.
- [x] gen-m7/59cc50701f4937c9 A24, gen-m7/93976dd9a3578ed9 A11 and (with T-1361) gen-m7/9ff3afcacd3bd0fc A35 pass. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEXTEST/EXTRACT/REPLACE lift their text and scalar arguments over ranges and
arrays; the delimiters stay lists (TEXTBEFORE/AFTER take a delimiter array as a list; a delimiter range is still one
text until T-1361). `lift_text_call` calls `lift_values` directly; moving onto `lift_call` is T-1775.
- Bounds (DEVIATION, text.md "Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* calls are capped"): the work is every
  operand element's length x its broadcast repeats, the text's further x the delimiter comparisons per position
  (delimiter count + compared bytes / 64, / 8 for TEXTSPLIT's re-searches), capped at 2^27 lifted or not. REGEX
  patterns compile once each under the smallest of five size limits they fit (64 KiB .. 10 MiB), within a per-call
  compile budget (128 MiB of limits) and a held cap (64 MiB, no eviction); every search is charged text bytes x size
  limit against 2^42 a call; a REGEXREPLACE result is sized before it is built (over 32,767 units `#VALUE!`, UNVERIFIED,
  T-1817 probes it).
- Performance review fixes (fifth run, release timings): `\w{200}b` over 4,000 x 3,000 bytes 38.5 s -> `#ERROR!` at
  1.1 s; TEXTSPLIT with a 16,000-byte delimiter matching everywhere 19 s -> `#ERROR!` at once (2 cells: 0.2 s, one
  `memchr::memmem::Finder` per delimiter per call); 5 cycling `\w{200}` patterns `#ERROR!` at 2.3 s -> 100 values at
  0.6 s; `ROWS(REGEXREPLACE(REPT("a",32767)&SEQUENCE(1000),"","b"))` `#ERROR!` at 1.0 s. New direct dependencies
  `memchr` and `regex-automata` (both already in the tree through `regex`).
- Performance review fixes (fourth run): delimiter lists folded once per call and borrowed per element; TEXTSPLIT's
  split searches each delimiter forward once instead of per piece; a lifted TEXTSPLIT/REGEXEXTRACT element computes only
  its first value (this absorbs T-1814).
- Unverified guesses are listed in text.md and the tests pinning them say so; T-1824 probes them.
- Eval: probes known failures -30 (rows 13, 15-22, 55 and K11:K13 of probes/m7-argument-modes), 0 regressions; gen-m7
  59cc50701f4937c9 A24, 93976dd9a3578ed9 A11, 9ff3afcacd3bd0fc A35 and 6db767d720c5b55a A28 now pass, no other
  gen-suite cell changed (checked against main's binary). Duplication 5561 -> 5375 against the current main (decision 0010).
- Handed off three times (budget), resumed by opus: every review finding is addressed in the branch history (the
  fourth run fixed the performance review's four items; release timings in the PR). The diff outgrew size S (about
  1,400 lines, much of it moving TEXTBEFORE/TEXTAFTER/TEXTSPLIT out of text/common.rs). Still to consolidate: the
  one-cell-range collapse (`collapse_single_cell`) inlined in several lift callers, and `lift_text_call` itself, both
  for T-1775.
  Follow-ups: T-1824 (probe the guesses), T-1817 (probe REGEXREPLACE past 32,767; the bound is done here); T-1814 done
  here.
