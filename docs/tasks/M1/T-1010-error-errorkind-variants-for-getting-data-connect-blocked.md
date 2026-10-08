---
id: T-1010
title: "Error/ErrorKind variants for #GETTING_DATA, #CONNECT!, #BLOCKED!, #FIELD! (SPEC-0003 E1)"
milestone: M1
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0177]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
The engine and the public API represent `#GETTING_DATA`, `#CONNECT!`, `#BLOCKED!` and `#FIELD!` as their own error kinds, with Excel's ERROR.TYPE codes, and `=#GETTING_DATA` parses.

## Context
SPEC-0003 "Design", sections "Engine" and "Facade and Python" (rollout E1). The fixed cells are `probes/error-literals-new` and `error-literals-new2`. Cells whose expected value is the stored `#N/A` pass only after T-1011, so count them there.
Code: `crates/rustsheet-engine/src/expressions/token.rs` (`enum Error`, `error_type_code`, `get_error_by_english_name`, `is_english_error_string`), `expressions/lexer/mod.rs` (`consume_error`), `crates/rustsheet/src/value.rs` (`ErrorKind`), and `bindings/python` tests.

## Acceptance criteria
- [ ] New `Error` variants `GETTINGDATA`, `CONNECT`, `BLOCKED`, `FIELD` are appended after `NULL`. `error_type_code` gives 8/10/11/13, and no `_` match arm is added anywhere.
- [ ] The stored-name table has all sixteen names. The formula-literal table adds only `#GETTING_DATA`, case-insensitively, without touching `Language`, and `=#BLOCKED!` still fails to parse. Unit tests cover both tables.
- [ ] `is_english_error_string` (COUNTIF-family criteria) is unchanged, with a test that `"#BLOCKED!"` is still not an error criterion. Tests for `Model::set_user_input` and `value_needs_quoting` pin the spec's choice: typed `#GETTING_DATA` is an error, typed `#BLOCKED!` is text.
- [ ] The `error_type_code` comment keeps 12 (`#UNKNOWN!`) and 19 (`#EXTERNAL!`) as "no evidence of a producer". UPSTREAM.md records the divergence from IronCalc's `Error` enum and lexer.
- [ ] `Error` gets no `#BUSY!` kind here; stored `#BUSY!` stays `#ERROR!` on import until T-1020 decides.
- [ ] `Error::stored_literal()` exists (`#N/A` for `GETTINGDATA`) with a unit test. Its callers come in T-1011.
- [ ] `ErrorKind` gains `GettingData`, `Connect`, `Blocked` and `Field`. `literals_match_engine` covers them, and `from_literal` uses the stored-name table and round-trips all kinds. A Python test builds `XlError("#BLOCKED!")` with code 11.
- [ ] The ERROR.TYPE/ISNA/IFNA/TYPE/VALUETOTEXT cells of `probes/error-literals-new` (A2–A6, A12, A14, A16) pass. They are recorded with `make eval-baseline`, with no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMAGE/STOCKHISTORY/FIELDVALUE (T-1013) and xlsx export (T-1011).

## Outcome
The engine `Error` enum has `GETTINGDATA`, `CONNECT`, `BLOCKED` and `FIELD`, appended after `NULL` with ERROR.TYPE 8/10/11/13. The stored-name table has all sixteen names. `#GETTING_DATA`, in any case, is the only new formula and typed-input literal. `Error::stored_literal` is added; T-1011 uses it.

The facade `ErrorKind` has the four new kinds, and `from_literal` uses the stored-name table, so Python `XlError("#BLOCKED!")` has code 11. UNIQUE now keys errors by `error_type_code` instead of a duplicate table. COUNTIF criteria names are unchanged (T-1012). Typed input follows SPEC-0003's UNVERIFIED choice (T-1016).

The xlsx importer still keeps cached `#CONNECT!`/`#BLOCKED!`/`#FIELD!` as `#VALUE!` (`TODO(T-1011)`), so that a round trip never writes literals Excel refuses.

Eval: probes known failures 290 -> 260 (30 fixed: error-literals-new 10, error-literals-new2 15, error-stored-constants 5), no regressions. The remaining error-literals cells expect the stored `#N/A` (T-1011).
