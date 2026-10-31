---
id: T-3384
title: "Formula import, part 4: cut the relocation key's lexing cost (KeyLexer::key, about 4G instructions on corpus-bench 52450)"
milestone: M11
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3326]
---

## Goal
Loading corpus-bench/52450 spends at most half of today's instructions in `KeyLexer::key`
(crates/rustsheet-xlsx/src/import/relocation_key.rs), with every imported formula's R1C1 text unchanged.

## Context
T-3326 skips the parse of copied formulas by a relocation key. Callgrind of
`rustsheet-eval run --suite corpus-bench --filter 52450` (release) puts `from_a1_to_rc` at 4.13G instructions, 4.05G of
them in `KeyLexer::key`: about 110k instructions per formula for a full re-lex, a `Vec<char>` of the formula, a
`peek_token` (a second lex) per identifier, an upper-cased copy per identifier and `to_string` per coordinate.
Candidates: lex once and reuse the token's position for the peek, look the function up without allocating, write
coordinates with `itoa`-style formatting into the key, or skip the key for a formula whose text equals the previous
formula's text shifted (a cheaper pre-check). Soundness arguments in the module docs must still hold.

## Acceptance criteria
- [x] Callgrind's `KeyLexer::key` count on corpus-bench/52450 before and after in the Outcome, after at most half.
- [x] `relocation_key_gives_equal_trees_for_equal_keys` and `rc_text_cache_prints_what_a_fresh_parse_prints` pass
      unchanged; `make eval-check` with corpus-bench and poi fetched shows no regression.
- [x] `make check` green; reviewers approve.

## Out of scope
- The argument-coverage census's own parse (`observe`, crates/rustsheet-eval).

## Outcome
- **Callgrind,** `rustsheet-eval run --suite corpus-bench --filter 52450`, release:

  | function (inclusive) | before | after |
  |---|---|---|
  | `KeyLexer::key` | 4.10G | 0.081G (−98%) |
  | `from_a1_to_rc` | 4.18G | 0.156G |
  | whole run | 22.9G | 19.7G |

  52450 now loads in 152 ms, from 545 ms (standalone `load_from_xlsx`). Other loads: calipers pv26_010
  565 → 490 ms, pv26_001 629 → 470 ms, pv23_057 616 → 557 ms. pv26_002 goes 3.70 → 3.41 s; its load is spent
  elsewhere.
- **Shifted copies take the template's key** (relocation_key.rs, soundness in the module docs). After keying a
  formula, `KeyLexer` keeps it as a template: its text, cell, key, and the span and value of each relative
  coordinate.
  - A later formula whose text is the template's with every relative coordinate moved by the cell offset, written
    canonically, and every other byte equal, takes the template's key without being lexed.
  - A template is kept only when that reading is certain. The text is ASCII. Each reference or range corner
    written in it must match its token's values and `$`s, including column and row ranges in their fixed forms.
    No range may have corners that differ in `$` on an axis.
  - Moved coordinates must stay on the grid. Anything else is lexed as before.
  - 52450's 36,640 copies of one formula, filled across columns over `'[1]XA Data Pull'!$J:$J`, now lex once.
- **Lexer allocations** (engine, every A1 parse benefits), each with the same tokens, error messages and positions:
  - `consume_reference_a1` folds the column and row as it reads them, instead of building two strings. It is
    split into a non-moving `scan_reference_a1`, so `consume_range_a1`'s try at a cell builds no error before
    falling back to a column range.
  - The row/column range branch folds its corners the same way (`corner_parts`). Its overflow error still
    quotes the digits.
  - `consume_number` reuses a buffer on the lexer and compares the decimal separator without a string.
  - The per-letter column step `push_column_letter` is shared with `column_to_number`.
- **Key building:** coordinates are written without `to_string`. `LET`/`LAMBDA` are compared without upper-casing.
  Stored-function answers are kept for the last 8 identifiers. The lexer's own chars are read instead of a copy.
- **Tests:**
  - lexer `reference_tokens_are_unchanged`: 96 formulas' tokens, positions and error messages, recorded from the
    lexer before this change and diffed against it. The cases cover off-grid and overflowing columns and rows,
    lower case, `$`, prefixes, ranges, and row and column ranges with their error texts;
  - `shifted_copies_take_the_keys_a_fresh_lexer_gives`: 12 formulas copied over 11 cells, plus text the engine
    never prints and off-grid moves. Each template key is compared with a fresh lexer's key, and a counter checks
    that templates served the copies;
  - `relocation_key_gives_equal_trees_for_equal_keys` and `rc_text_cache_prints_what_a_fresh_parse_prints` pass
    unchanged.
- **Mutations that fail a test:** dropping the mixed-`$` check, the row bounds, or the column validity check; the
  wrong error text for an empty column. Two survive, both unobservable through the lexer: the "at least one digit"
  filter (its error is always re-wrapped by callers) and the corner-form check (no token has corners of
  different forms). Both are kept as guards.

