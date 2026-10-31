---
id: T-3384
title: "Formula import, part 4: cut the relocation key's lexing cost (KeyLexer::key, about 4G instructions on corpus-bench 52450)"
milestone: M11
lane: perf
status: todo
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
- [ ] Callgrind's `KeyLexer::key` count on corpus-bench/52450 before and after in the Outcome, after at most half.
- [ ] `relocation_key_gives_equal_trees_for_equal_keys` and `rc_text_cache_prints_what_a_fresh_parse_prints` pass
      unchanged; `make eval-check` with corpus-bench and poi fetched shows no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The argument-coverage census's own parse (`observe`, crates/rustsheet-eval).
