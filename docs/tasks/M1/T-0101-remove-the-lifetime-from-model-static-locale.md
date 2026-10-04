---
id: T-0101
title: "Remove the lifetime from Model (static locale/language data)"
milestone: M1
lane: engine
status: in-progress
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
