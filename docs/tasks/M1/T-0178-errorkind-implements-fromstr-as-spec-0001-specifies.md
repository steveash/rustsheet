---
id: T-0178
title: "ErrorKind implements FromStr as SPEC-0001 specifies"
milestone: M1
lane: api
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet::ErrorKind` implements `std::str::FromStr`, as SPEC-0001 ("literal round-trips through
`ErrorKind::literal()` / `FromStr`") specifies, so callers can write `"#N/A".parse::<ErrorKind>()`.

## Context
T-0113 shipped `ErrorKind::from_literal(&str) -> Option<Self>` (crates/rustsheet/src/value.rs) and
listed "no `FromStr`" as a deviation from SPEC-0001, but the spec text (docs/specs/SPEC-0001-public-api-v0.md,
"Error values keep their Excel literal") was never changed, so spec and code disagree. Implement
`FromStr` by delegating to `from_literal` (one definition, CLAUDE.md directive 3); its `Err` is a
new public error type made with `thiserror`, with a doc comment, holding the rejected text.
Keep `from_literal` (the Python binding's `XlError` constructor in bindings/python/src/facade.rs uses it).

## Acceptance criteria
- [ ] `impl FromStr for ErrorKind` with a `thiserror` error type exported from the crate root; doc
      comments on both.
- [ ] Unit test in value.rs: every kind round-trips `kind.literal().parse::<ErrorKind>() == Ok(kind)`,
      lower case parses as `from_literal` does, `"#BOGUS!".parse::<ErrorKind>()` is `Err` and its
      message contains `#BOGUS!`.
- [ ] SPEC-0001's sentence matches the code (mentions both `from_literal` and `FromStr`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- New error kinds (T-0177).
- Changing `from_literal` or the Python binding.

## Outcome
`ErrorKind` implements `FromStr` (delegating to `from_literal`) with `ParseErrorKindError`; SPEC-0001 sentence updated; unit test added. No eval delta.
