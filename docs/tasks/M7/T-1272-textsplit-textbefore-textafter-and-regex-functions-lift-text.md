---
id: T-1272
title: "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX functions lift text over a range and scalar arguments over an array"
milestone: M7
lane: functions-text
status: in-progress
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
- [ ] Probe rows 13, 15-22 and 55 and cells K11-K13 pass; baseline updated.
- [ ] gen-m7/59cc50701f4937c9 A24, gen-m7/93976dd9a3578ed9 A11 and (with T-1361) gen-m7/9ff3afcacd3bd0fc A35 pass. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEXTEST/EXTRACT/REPLACE lift their text and scalar arguments over ranges and arrays; the delimiters stay lists (TEXTBEFORE/AFTER now take a delimiter list). Static analysis marks them array-capable. Eval delta: probes known failures 3072 -> 3042 (30 cells fixed, 0 regressions); gen-m7 A24/A11 now `#SPILL!` as in Excel. Unverified guesses are listed in docs/excel-semantics/text.md.

## Handoff (resumed)
Branch `agent/functions-text/T-1272-text-lift` has the full lift implementation (text/before_after.rs, text/lifted.rs, regex_functions.rs, textsplit.rs, util.rs multi-delimiter matching, static_analysis marks the six functions array-capable); `make check` was green before the review, the probe/gen-m7 cells pass and the baseline is updated (probes 3072 -> 3042). The review round returned REQUEST_CHANGES from security, quality and performance (architecture APPROVE, excel-fidelity not yet reported):
- MAJOR (security, performance): TEXTBEFORE/TEXTAFTER now take an unbounded delimiter list scanned per character (`util.rs` `delimiter_at`); a NON-lifted call such as `TEXTBEFORE(REPT("a",32767),SEQUENCE(1048576))` costs ~3e10 prefix checks and `lift_values` never applies the visit cap when nothing lifts. Needs a cost check (text length x delimiters) for the single call too, or a multi-pattern matcher; TEXTSPLIT's `split_by_any` has the same pre-existing exposure. Document as DEVIATION (decision 0014).
- MAJOR: REGEX* lifting the pattern can compile many distinct patterns; `PatternCache` keeps only the last. I already added a size-aware cost to `lift_text_call` (longest text x delimiter count, plus `COMPILE_COST` when the pattern lifts) in the working tree; still to do: a bounded multi-entry cache, small `RegexBuilder::size_limit`, fix the doc comment/text.md wording ("about 100,000 elements" is no longer right).
- MAJOR (quality): unit tests for multi-delimiter helpers (added `delimiter_lists` in util.rs) and focused tests (added in test/text_functions/fn_textbefore.rs); the over-the-cap test I tried failed because a non-lifted call is never capped (the first MAJOR). Also split the ~100-line `text_around_element`, rewrite old comments to cite evidence, run `scripts/check_duplication.py --update` (6111 -> 5925).
- MINOR (performance): per-element clones of delimiter lists and `values[i]`, REGEXEXTRACT/TEXTSPLIT build the full array for a lifted element when only [0][0] is kept.
Next: finish the above, re-run `make check`, merge origin/main, re-run the failed reviewers plus excel-fidelity, open the PR.

### Excel-fidelity review (REQUEST_CHANGES)
- BLOCKING: gen-m7/851eb2099ae3f805 Calc!A25 `TEXTAFTER(" x ",Data!A6:A10,Data!A9)` passed on main (`#VALUE!`, as Excel) and now gives `#N/A`, because a multi-cell delimiter range is accepted early (T-1361's scope) and its blanks are dropped. gen-m7 has no baseline, so `make eval-check` misses it. Either keep ranges with blanks, numbers or booleans on the old `#VALUE!` path until T-1361 probes them, or probe and implement the rule (instance_num > LEN(text) may be `#VALUE!`, but gen-m9/4df1b22d3ed63f58 A14 clashes with it). Record the cell in text.md and T-1361.
- MAJOR: tests pin unproven behaviour (first-in-list tie-break, all entries longer than the text, an error inside the delimiter array, per-element regex compile error, if_not_found/pad_with/match_end/return_all lifting; if_not_found is now evaluated eagerly). Mark each as unverified in the test with a comment citing text.md, and file a probe task (excel-probe) linked from the entry's `Tasks:` line.
- MINOR: text.md "Ours:" line of the delimiter-ranges entry is out of date (multi-row text ranges now work; gen-m9 3ec66461400faf2f A3 and 95efb37a9874f82e A25 pass); the DEVIATION wording "about 100,000 elements" is wrong (cost is (longest text + 1) x delimiters, plus COMPILE_COST when the pattern lifts, against 2^27); TEXTSPLIT's first-value-of-lifted-element is VERIFIED by probe rows 13 (keep REGEXEXTRACT return_all unverified); comments cite text.md headings that do not exist (real heading: "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar arguments").
- Eval: `make eval-check` has 0 regressions; also fixed gen-m7 59cc50701f4937c9 A24, 93976dd9a3578ed9 A11, 9ff3afcacd3bd0fc A35, 6db767d720c5b55a A28 and gen-m9 3ec66461400faf2f A3, 95efb37a9874f82e A25.

### Resumed by opus 2026-10-17 (session budget ran out; status back to todo, tier opus)
Done on the branch, `make check` green:
- Merged origin/main (T-1770's SPEC-0011 driver). `lift_text_call` stays a direct caller of `lift_values`, like the *IF(S) and lookup callers. SPEC-0011 moves the text functions onto `lift_call` later (T-1775). It uses the new `LiftOptions::EVERY_ELEMENT`.
- BLOCKING fidelity: a multi-cell range as the TEXTBEFORE/TEXTAFTER delimiter is read as one text again (`around_delimiters`), so gen-m7/851eb2099ae3f805 A25 is `#VALUE!` as on main. gen-m9/3ec66461400faf2f A3 and 95efb37a9874f82e A25 fail again, as on main; T-1361 now cites all three.
- MAJOR (security, performance), first half: `lift_text_call` caps a call that does not lift by its own cost, so `TEXTBEFORE(REPT("a",32767),SEQUENCE(10000))` is `#ERROR!`. Test: `unlifted_call_over_a_long_delimiter_list_is_capped`. text.md's DEVIATION wording is fixed.
- Fidelity MAJOR, filing half: T-1812 (renumbered from T-1809, which main had taken) probes the guesses the tests pin; text.md links it.

Still open, in this order:
1. Mark each guess-pinning test as UNVERIFIED with a comment citing text.md and T-1812 (fidelity MAJOR).
2. REGEX: a bounded multi-entry `PatternCache`, a small `RegexBuilder::size_limit`, and the doc comment fix (MAJOR).
3. Quality: split the long `text_around_element`; rewrite old comments to cite evidence; run `scripts/check_duplication.py --update` and commit the drop.
4. MINOR performance: per-element clones of the delimiter lists and of `values[i]`; REGEXEXTRACT/TEXTSPLIT build the full array for a lifted element when only [0][0] is kept.
5. Re-run security, quality, performance and excel-fidelity, open the PR, and check the gen-m7/gen-m9 cells by hand: they have no baseline, so `make eval-check` misses them.
