---
id: T-2905
title: "Route the engine's non-test en-US locale and language lookups through get_default_locale and get_default_language"
milestone: M2
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine has one way to get the built-in en-US locale and language outside tests: `get_default_locale()`
(crates/rustsheet-engine/src/locale/mod.rs) and `get_default_language()` (crates/rustsheet-engine/src/language/mod.rs).
The ad-hoc `get_locale("en").expect("")` / `get_language("en").expect("")` / `.ok()?` lookups in non-test code
are gone, and with them five local `#[allow(clippy::expect_used)]` attributes.

## Context
Directive 3 (one definition per concept). The two default helpers already exist and are used at 127 call sites;
these non-test sites still repeat the lookup by hand (line numbers at 7bd9090e):
- crates/rustsheet-engine/src/expressions/parser/structured_reference.rs:136-139: `get_locale("en").expect("")`
  and `get_language("en").expect("")`, each under `#[allow(clippy::expect_used)]`.
- crates/rustsheet-engine/src/expressions/lexer/util.rs:53-56 (`get_tokens`): the same pair, two allows.
- crates/rustsheet-engine/src/utils.rs:58-59 (`parse_anchored_reference_formula`): `get_language("en").expect("")`,
  one allow.
- crates/rustsheet-engine/src/expressions/utils/shift.rs:19-20 (`shift_formula_text`): `get_locale("en").ok()?` and
  `get_language("en").ok()?`. "en" is compiled in and the lookup cannot fail, so the `None` branch is dead; the
  default helpers keep the function's `Option` result for real lex failures.

`#[cfg(test)]` modules (sequence_detector.rs, utils.rs, date_text.rs and the parser tests) and the module doc example
in expressions/lexer/mod.rs are out of scope: tests may `unwrap`. The helpers themselves keep their single
`#[allow(clippy::unwrap_used)]`; that is the one place the "en is always present" assumption lives.

## Acceptance criteria
- [ ] `rg -n 'get_(locale|language)\("en"\)' crates/rustsheet-engine/src` lists only the two default helpers,
      `#[cfg(test)]` code, files under `tests/` or `test/`, and the doc comment in expressions/lexer/mod.rs.
- [ ] The five `#[allow(clippy::expect_used)]` attributes at the sites above are deleted, and no new `allow` is added.
- [ ] No behaviour change: `make eval` shows 0 regressions and 0 fixes, and the existing unit tests pass unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locale or language selection for non-en workbooks (M10).
- Test code and the lint-debt allow-lists in Cargo.toml (`quality/lint-debt.json` is a protected path).
