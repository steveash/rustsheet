---
id: T-3574
title: "Formula import part 5: relocation-key templates for mixed-ranges and per column (calipers pv26_002 load)"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Loading calipers `pv26_002` keys most of its formula copies from a template (T-3384) instead of lexing
each one. That covers running totals and ranks (`SUM($I11:I11)`, `COUNTIFS(KP$11:KP22,KP23)`), and copies
read with other formulas between them. Every key stays the one a fresh lex gives.

## Context
After T-3471, `pv26_002` loads in about 3.4 s. Stack samples (2026-10-31, 134 during load) put 41% of
them in `KeyLexer::key` and `from_a1_to_rc`. Of its 680,000 formulas keyed, only about 26% came from a
template. Two reasons:
- `range_coordinates` refused every range whose corners differ on `$` on an axis. Those are the
  running-total, rank and prefix idioms, `KP$11:KP22` and `$I47:I47`. The refusal is there because the key
  orders a range's corners by value (`<` or `>` per axis), and a shift can flip that order when one corner
  is relative and the other absolute.
- `KeyLexer` kept one template, the last formula keyed. A sheet's formulas come row by row, so a column's
  copy usually has the row's other formulas between it and its template.

## Acceptance criteria
- [x] A range whose corners differ on `$` on an axis is kept in a template, with the comparison of those
      corners. A copy takes the template's key only where its moved corners compare alike.
- [x] The last template of each column is kept, at most `KeyLexer::TEMPLATES` (directive 5). The cell's
      column is tried first, then the last template kept.
- [x] Tests:
      - `shifted_copies_take_the_keys_a_fresh_lexer_gives` gains mixed-`$` ranges whose order flips in
        some copies.
      - `templates_are_kept_per_column` covers interleaved fills.
      - The random-text and frozen-token tests pass unchanged.
- [x] calipers `pv26_002` load time before and after, in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Templates for non-ASCII formulas, structured references, or a lexer-free key.

## Outcome
- **Corner order:** `Order` records, for a template's range whose corners differ on `$` on an axis, each
  corner's value, whether it is relative, and how they compare. `Template::key_of_copy` checks every order
  still holds at the copy's offset before comparing the shifted text, because the key's `<`/`>` is that
  comparison.
- **Per-column templates:** `KeyLexer` keeps `templates: HashMap<column, Template>` (at most 1,024, cleared
  when full) and the column of the last one kept. `forget_template` drops the last one kept, and a formula
  that is no template drops its column's.
- **Timing:** calipers `pv26_002` release load went from 3.3–3.7 s to 2.6–2.8 s. The mixed-`$` orders
  alone gave 2.88 s, and per-column templates took off the rest. Eval time is unaffected.
- **Mutations caught:**
  - no order check;
  - orders never recorded;
  - the offset applied to the absolute corner;
  - the row and column offsets swapped;
  - no per-column lookup.
- **Review fixes:**
  - `forget_template` drops only the template the failing formula's own `key` call kept (`kept_now`), not
    another column's. A test checks this.
  - `Order::new` replaces the inline construction.
  - The module docs are re-wrapped.
