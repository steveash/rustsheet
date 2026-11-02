---
id: T-3820
title: "Probe which character after a capital sigma makes LOWER give the medial σ: Excel's letter table against Rust's is_alphanumeric"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence decides, for each class of character that can follow a capital `Σ`, whether LOWER (and PROPER,
which shares the rule) lowers that `Σ` to the medial `σ` or the final `ς`. text.md then states the predicate
Excel uses, and every class where the engine disagrees has an owner task.

## Context
- The rule today (docs/excel-semantics/text.md "UPPER, LOWER and PROPER: one-to-one case mapping, final-sigma
  context (T-1747)"): `Σ` lowers to `ς` before the end of the text, a space, `'` or `-`, and to `σ` before a letter
  or digit, a combining mark (only U+0301 and U+0308 probed), `’` (U+2019) or a zero-width space (U+200B). The same
  entry says "other combining blocks and U+200C/U+200D/U+2018 are unprobed and give ς". Evidence:
  probes/case-mapping-expanding S!C15:C27 (excel-graph, 2026-10-17).
- The engine's predicate is `sigma_is_final` (crates/rustsheet-engine/src/functions/text/string_format.rs): `σ`
  when the next character is `char::is_alphanumeric()`, U+0300-U+036F, U+2019 or U+200B. `is_alphanumeric` is
  Rust's current Unicode `Alphabetic || Numeric`. T-3298 (text.md "UPPER and LOWER use an old per-character case
  table") showed Excel's own character table is far older than current Unicode: it leaves hundreds of letters
  unmapped (Cherokee small letters, Georgian Mtavruli, Latin Extended-D, every supplementary-plane letter). Whether
  the sigma test reads that old table, a word-break table, or something else is unknown, and the engine's answer
  for every letter Excel's table lacks is a guess.
- `Numeric` also takes in characters that are not digits (`²`, `½`, `Ⅻ` U+216B, `①` U+2460) and script digits
  (`٣`, `३`, full-width `０`); none is probed after a `Σ`.
- The same predicate drives `fold_for_before_after` (functions/text/casefold.rs, TEXTBEFORE/TEXTAFTER match_mode 1)
  and `proper_case`, so the answer moves three functions.
- Lead (not evidence): docs/research/holdout-themes-2026-11-01.md lists "PROPER and LOWER: some non-ASCII Unicode
  case mappings differ from Excel's". T-3583 owns the mapping table, T-3657 PROPER's word starts; neither probes
  the sigma context.
- Layout suggestion (one numeric reader per row, so a cell is unambiguous): Data!A`n` holds `"ΑΣ"&UNICHAR(x)` (or
  the text built with UNICHAR in a formula), Calc!A`n` `=UNICODE(MID(LOWER(Data!A<n>),2,1))` (962 = `ς`, 963 = `σ`),
  Calc!B`n` the same over PROPER, Calc!C`n` `=IFERROR(LEN(TEXTBEFORE("x"&Data!A<n>,"σ",1,1)),-1)`.
  Fill with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes` (excel-probe skill).

## Acceptance criteria
- [ ] `eval/suites/probes/cases/t3820-lower-sigma-next-char.case.json` and its `.oracle.json` sidecar (excel-graph)
      are committed. At least one row per class: letters Excel's case table lacks (U+0526, U+10C7, U+13A0, U+AB70,
      U+1C90, U+A7C0, U+10400 `𐐀`); uncased letters (`א`, `ا`, `ก`, `あ`, `一`, `가`); modifier letters (`ʰ`
      U+02B0, `ª`); non-digit numerics (`²`, `½`, U+216B, U+2460); script and full-width digits (`٣`, `३`, `０`);
      combining marks outside U+0300-U+036F (U+0483, U+05B0, U+093E, U+0E48, U+1AB0, U+1DC0, U+20D0, U+FE20); format
      characters (U+00AD, U+200C, U+200D, U+2060, U+FEFF, U+FE0F) and U+2018; an emoji (`😀`); and `Σ`, `σ`, `ς`
      themselves. Each row is read through LOWER, PROPER and TEXTBEFORE as above.
- [ ] text.md's T-1747 entry replaces "other combining blocks and U+200C/U+200D/U+2018 are unprobed and give ς"
      with the observed rule, citing the rows; a class whose answer the probe cannot explain stays marked UNVERIFIED.
- [ ] The cells the engine fails are known failures in eval/baseline/probes.txt (`make eval-baseline`, 0
      regressions), and a fix task is filed with `make new-task` (M2, functions-text) listing them (case file stem
      and A1 names in one bullet, decision 0159), so `make exit-owners MILESTONE=M2` still reports 0. If the
      engine already matches every row, the Outcome says so and no task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `sigma_is_final` or any engine code (the filed follow-up).
- UPPER and LOWER's mapping table (T-3583) and PROPER's word starts (T-3657).
- What precedes the `Σ` (verified by T-1747: no preceding letter is needed).
