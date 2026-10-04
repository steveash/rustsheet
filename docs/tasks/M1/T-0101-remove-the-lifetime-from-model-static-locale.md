---
id: T-0101
title: "Remove the lifetime from Model (static locale/language data)"
milestone: M1
lane: engine
status: done
priority: P1
size: M
depends_on: []
---

## Goal
Make `Model` own or reference `'static` locale/language data so public types have no lifetimes and the Python binding stops leaking strings.

## Context
docs/research/ironcalc-assessment.md §6/§8: Model<'a> borrows &'a Locale/Language; bindings use a leak hack.

## Acceptance criteria
- [ ] `Model` and `UserModel` have no lifetime parameter; all crates and bindings compile; leak helper removed from bindings.
- [ ] `make check` green, eval unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
Other API changes.

## Outcome
`Model`, `UserModel`, `Parser` and `Lexer` no longer carry a lifetime; locale/language references are
`&'static` (the data was already static). The Python binding's `leak_str` helper is removed. Eval unchanged
(0 fixed, 0 regressions).
