---
id: T-1222
title: "Stored #BUSY! error kind (ERROR.TYPE 0): engine, facade and xlsx per the SPEC-0003 amendment"
milestone: M1
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1020, T-1221]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
A stored `#BUSY!` error constant behaves as in Excel, both from a case spec and from an xlsx file.
The known-failure cells of the frozen M1 probe `probes/error-stored-constants` pass (S!B1, D1, H1
and I1 in eval/baseline/probes.txt after T-1221; 7 cells before it).

## Context
Expected and actual values (`make eval-show CASE=probes/error-stored-constants.case.json`):

| cell | formula | Excel | RustSheet today |
|---|---|---|---|
| B1 | `=ERROR.TYPE(A1)` | 0 | #N/A |
| C1 | `=ISERROR(A1)` | TRUE | FALSE |
| D1 | `=A1&""` | #BUSY! | "#BUSY!" |
| E1 | `=ISERR(A1)` | TRUE | FALSE |
| G1 | `=TYPE(A1)` | 16 | 2 |
| H1 | `=ERROR.TYPE(A1)+100` | 100 | #N/A |
| I1 | `=IFNA(A1,"y")` | #BUSY! | "#BUSY!" |

The table shows actual values from before T-1221. After T-1221, A1 loads as `#ERROR!`, and some of
these cells may already pass.

Semantics: docs/excel-semantics/syntax.md, "Newer error literals" (T-0999). Follow the design that
T-1020 writes into SPEC-0003. The code to touch is the same set T-1010 changed for the four newer
kinds:
- `crates/rustsheet-engine/src/expressions/token.rs`: `enum Error`, `error_type_code`, the
  stored-name table, `stored_literal`.
- `crates/rustsheet/src/value.rs`: `ErrorKind`, `literals_match_engine`.
- `crates/rustsheet-xlsx/src/import/worksheets.rs`.
- `crates/rustsheet-xlsx/src/export/worksheets.rs`: `exported_error_literal`.
- Python binding tests.
- UPSTREAM.md, for the divergence from IronCalc's `Error` enum.

## Acceptance criteria
- [x] probes/error-stored-constants passes in full (24/24 cells). Its F lines leave
      eval/baseline/probes.txt via `make eval-baseline`, with no new F lines in any suite.
- [x] The kind follows SPEC-0003 as amended by T-1020. No `_` match arm is added. Unit tests show
      that `=#BUSY!` still fails to parse, and that typed `#BUSY!` cell input and
      `is_english_error_string("#BUSY!")` are unchanged, unless the amended spec says otherwise.
- [x] A unit test loads an xlsx with a stored `<c t="e"><v>#BUSY!</v></c>` and gets the new kind
      (`ERROR.TYPE` 0). Export writes what the spec decides, with a test.
- [x] The facade `ErrorKind` round-trips the kind through `from_literal`/`literal`, and a Python test
      covers `XlError("#BUSY!")`.
- [x] A unit test runs UNIQUE over cells holding `#BUSY!` and `#NULL!` (code 0 is its own
      `CellKey::Error` key; SPEC-0003 "Stored `#BUSY!`"). This pins the engine; Excel is T-1228.
- [x] syntax.md "Newer error literals" gets an `Implemented in:` line naming the code.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any formula that produces `#BUSY!` (none is known).
- `#UNKNOWN!`, `#PYTHON!` and `#EXTERNAL!`.

## Outcome
`Error::BUSY` (appended after `FIELD`, ERROR.TYPE 0) and `ErrorKind::Busy` (`code() == Some(0)`)
per SPEC-0003 "Stored `#BUSY!`". Only the stored-name table (`get_error_by_english_name`) gains
`#BUSY!`: `=#BUSY!` still fails to parse, typed `#BUSY!` stays text, and `"#BUSY!"` stays a text
criterion. xlsx export writes the plain `<v>#BUSY!</v>` (no `vm`) and import reads it back as the
kind. Tests: token/lexer units, engine probe-cell and UNIQUE (`#BUSY!` vs `#NULL!` stay apart)
tests, xlsx round trip, facade and Python `XlError("#BUSY!").code == 0`, eval adapter.
Eval delta: probes 561 → 557 known failures (probes/error-stored-constants 24/24), 0 regressions.
