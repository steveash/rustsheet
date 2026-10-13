# 0033 — Engine text cannot hold lone surrogates

- **Status:** superseded by decision 0052 (2026-10-13). Lone surrogates are a known gap, not a
  deviation; T-1483 specifies engine text that can hold them. Accepted 2026-10-06, T-0233.
- **Context:**
  - Excel text is UTF-16. Operations that cut text at a code-unit position can leave an unpaired
    surrogate, and Excel keeps it: a concatenation truncated at the 32,767-unit limit inside an
    emoji keeps the high surrogate (`LEN(CONCATENATE(REPT("a",32766),"😀"))` is 32767, and
    `UNICODE(RIGHT(…,1))` of it is `#VALUE!`; probes/concatenate-limit-spill E8–E11, excel-graph).
  - Engine text is a Rust `String` (UTF-8), which cannot hold an unpaired surrogate. Moving every
    value, the parser and the xlsx layer to UTF-16 or WTF-8 strings is far larger than the cases
    it would fix.
  - This is not a directive 5 (untrusted input) deviation, so decision 0014 does not cover it.
- **Options considered:**
  - *Drop the straddling pair whole.* The result is one unit shorter than Excel's; no character
    Excel did not produce appears.
  - *Replace the lone surrogate with U+FFFD.* Lengths match Excel, but the text then holds a
    character Excel never has, and `UNICODE`/`CODE`/comparisons on it differ anyway.
  - *UTF-16 or WTF-8 engine strings.* Exact, but an architecture change touching every text path.
- **Decision:** wherever Excel would split a surrogate pair, the engine drops the whole pair (the
  cut moves back one unit). Such cells are documented as `Status: DEVIATION (decision 0033)` in
  docs/excel-semantics and stay known failures.
- **Consequences:**
  - `cast::truncate_text` (CONCATENATE, `&`) follows it now; LEFT/MID/RIGHT/REPLACE positions that
    split a pair follow the same rule when they count UTF-16 units (T-0249).
  - probes/concatenate-limit-spill E8–E11 stay in eval/baseline as known failures.
  - Revisit if the engine's string type ever changes (e.g. for performance or xlsx round-trip of
    unpaired surrogates).
