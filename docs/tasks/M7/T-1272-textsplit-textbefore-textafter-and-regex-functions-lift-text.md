---
id: T-1272
title: "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX functions lift text over a range and scalar arguments over an array"
milestone: M7
lane: functions-text
status: todo
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

## Outcome (draft; not merged)
TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEXTEST/EXTRACT/REPLACE lift their text and scalar arguments over ranges and
arrays; the delimiters stay lists (TEXTBEFORE/AFTER take a delimiter array as a list; a delimiter range is still one
text until T-1361). `lift_text_call` calls `lift_values` directly; moving onto `lift_call` is T-1775.
- Bounds (DEVIATION, text.md "Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* calls are capped"): the work is every
  operand element's length x its broadcast repeats, the text's further x the delimiter comparisons per position
  (delimiter count + compared bytes / 64), capped at 2^27 lifted or not. REGEX patterns compile under the smallest of
  64 KiB / 1 MiB / 10 MiB size limits they fit, with a per-call compile budget (256 MiB of limits) and an LRU cache
  bounded at 32 MiB of limits.
- Performance review fixes (fourth run): delimiter lists folded once per call and borrowed per element; TEXTSPLIT's
  split searches each delimiter forward once instead of per piece; a lifted TEXTSPLIT/REGEXEXTRACT element computes only
  its first value (this absorbs T-1814).
- Unverified guesses are listed in text.md and the tests pinning them say so; T-1816 probes them.
- Eval: probes known failures -30 (rows 13, 15-22, 55 and K11:K13 of probes/m7-argument-modes), 0 regressions; gen-m7
  59cc50701f4937c9 A24, 93976dd9a3578ed9 A11, 9ff3afcacd3bd0fc A35 and 6db767d720c5b55a A28 now pass, no other
  gen-suite cell changed (checked against main's binary). Duplication 6111 -> 5925.
- Handed off three times (budget), resumed by opus: every review finding is addressed in the branch history (the
  fourth run fixed the performance review's four items; release timings in the PR). The diff outgrew size S (about
  1,400 lines, much of it moving TEXTBEFORE/TEXTAFTER/TEXTSPLIT out of text/common.rs). Still to consolidate: the
  one-cell-range collapse (`collapse_single_cell`) inlined in several lift callers, and `lift_text_call` itself, both
  for T-1775.
  Follow-ups: T-1816 (probe the guesses), T-1817 (REGEXREPLACE result bound); T-1814 done here.

## Handoff (resumed)
Fourth opus run (2026-10-17, budget out). Branch merged with main; `make check` green. PR #1016 (draft). This run fixed
the previous performance review's four items (fold lists once, delimiter-byte cost, tiered regex compile budget,
first-value-only lifted elements; absorbs T-1814), the quality review's two MAJORs (`LiftedArg::read`, `grid_dims`
shared with the lift driver) and the governance gate (decision 0010 entry). Review round on 877196fa: security,
architecture and quality APPROVE (quality after the shared-helper fix); excel-fidelity's MAJOR (a 1x1 array now lifts:
`COLUMNS(TEXTSPLIT({"a,b"},","))` 2 → 1) is recorded UNVERIFIED in text.md, pinned by
`fn_textsplit::one_by_one_array_lifts` and listed first in T-1816, so re-run excel-fidelity to confirm that suffices.
Performance REQUEST_CHANGES, still open (measured with a release harness):
1. BLOCKING: regex search cost is not counted. `=SUM(--REGEXTEST(REPT("a",3000)&SEQUENCE(4000),"\w{200}b"))` takes
   38.5 s at about 9% of the cap (the lazy DFA gives up on big Unicode classes). Fix: in `PatternCache::get`'s caller,
   charge `text.len() x weight(size tier)` against a per-call search budget, `#ERROR!` when spent (DEVIATION); measure
   the weights per tier; add a regression test.
2. BLOCKING: TEXTSPLIT re-searches cost more than the model says: `rest.find(d)` builds a new searcher per piece, so
   `=ROWS(TEXTSPLIT(REPT("a",32767)&SEQUENCE(16),HSTACK("a",REPT("a",16000))))` takes 19 s. Fix: one searcher per
   delimiter per call (`memchr::memmem::Finder`, or `aho-corasick` leftmost-first; both already in Cargo.lock through
   `regex`, so adding one as a direct dependency needs the security reviewer's OK), or charge the re-search by needle
   length.
3. MAJOR: the LRU cache still loses to cycling large patterns: `=ROWS(REGEXTEST("a","\w{"&(200+MOD(SEQUENCE(100),5))&"}"))`
   is `#ERROR!` after 2.3 s with only 5 distinct patterns. Fix: visit elements grouped by pattern (compile each distinct
   pattern once), budget distinct patterns; test with more cycling large patterns than the cache holds (release-only
   timing; a debug unit test can use mid-tier patterns such as `\w{2}`..`\w{16}`).
4. MAJOR: lifting multiplies the REGEXREPLACE result blow-up: count matches x replacement length before building
   (T-1817's fix) inside this PR, at least for the lifted path.
MINORs: `COMPILE_BUDGET` (about 2.3 s per cell) could be smaller; textsplit.rs `text.clone()` for the case-sensitive
comparison could be a `Cow`; `Delimiters::new` could fold lazily; add TEXTSPLIT/TEXTBEFORE/REGEX* cases to
crates/rustsheet/benches/text.rs with before/after numbers. Next: fix 1-4 with a regression test each, update the text.md
DEVIATION entry, `make check`, re-run performance and excel-fidelity, then mark done and merge.
