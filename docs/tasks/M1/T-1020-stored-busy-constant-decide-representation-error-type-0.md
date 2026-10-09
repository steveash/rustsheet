---
id: T-1020
title: "Stored #BUSY! constant: decide representation (ERROR.TYPE 0) per SPEC-0003"
milestone: M1
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: [T-1010]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
SPEC-0003 says how RustSheet represents a stored `#BUSY!` error constant, so that T-1222 can
implement it. This task is the spec amendment only. The planner split the implementation out on
2026-10-09, because a public `ErrorKind` change needs its spec first.

## Context
T-0999 found that Excel loads a stored `<c t="e"><v>#BUSY!</v></c>` as an error that is not
`#N/A`. Its `ERROR.TYPE` is 0, `TYPE` 16, `ISERR` and `ISERROR` TRUE, `ISNA` FALSE, `IFNA` passes it
through, and `A1&""` is `#BUSY!` (docs/excel-semantics/syntax.md, "Newer error literals"; evidence in
probes/error-stored-constants S!B1:I1).

SPEC-0003 left `#BUSY!` as a non-goal because no formula produces it. But
probes/error-stored-constants is on the frozen M1 probe list (eval/milestones/m1-probes.txt), and
its 7 failing cells (S!B1, C1, D1, E1, G1, H1, I1) block M1 exit. So the representation must be one
under which those cells can pass; "not supported" is no longer an option.

Today the eval adapter enters the stored constant as typed text (T-1221 fixes that). xlsx import
maps it to `#ERROR!` (`crates/rustsheet-xlsx/src/import/worksheets.rs`, `unwrap_or(Error::ERROR)`).

Questions the amendment must answer, citing evidence or marking UNVERIFIED:
- Is it a new `Error`/`ErrorKind` variant (appended after `FIELD`/`Field`, no `_` arms)? How do
  `error_type_code` (0) and the facade `ErrorKind::code()` (`Option<u8>`) report it, and does
  Python `XlError` expose code 0?
- Is it only in the stored-name table (`get_error_by_english_name`), and kept out of the
  formula-literal table, typed input (`set_user_input`) and COUNTIF criteria names? `=#BUSY!`
  makes Excel reject the workbook (syntax.md).
- What does xlsx export write? Excel loads a stored `#BUSY!` literal, so writing it back as is
  looks right, unlike `exported_error_literal`'s `#VALUE!` fallback.
- What does `VALUETOTEXT`, and leftmost-error precedence (arrays.md), give with it? These are
  unprobed. If the answer matters for the design, file a research probe; otherwise state the
  assumption.

## Acceptance criteria
- [x] docs/specs/SPEC-0003-newer-error-kinds.md is amended. `#BUSY!` moves out of "Non-goals" into
      "Excel behavior to match" and "Design", citing probes/error-stored-constants S!B1:I1. The
      rollout table row B1 points to T-1222 for the implementation.
- [x] Each question above is answered, with evidence or as an explicit UNVERIFIED assumption.
- [x] Follow-up research tasks are filed for any unprobed behavior the design depends on.
- [x] No code changes. `make check` green; reviewers approve.

## Out of scope
- The implementation (T-1222) and the eval adapter (T-1221).
- `#PYTHON!`, `#EXTERNAL!` and `#UNKNOWN!` (unprobed, or rejected when stored).

## Outcome
SPEC-0003 is amended with a "Stored `#BUSY!`" design section, a row in the behavior table (citing
probes/error-stored-constants S!B1:I1), and rollout rows B2 (T-1222) and R3 (T-1228). The decisions:
- A new `Error::BUSY`, appended after `FIELD`, with ERROR.TYPE 0.
- It goes in the stored-name table only. The formula, input and criteria tables stay unchanged:
  typed input and the criteria table are UNVERIFIED (T-1016, T-1228).
- Export writes `#BUSY!` as is.
- The facade has `ErrorKind::Busy` with `code()` `Some(0)`, and Python tests compare with `== 0`.
- VALUETOTEXT, precedence and IFERROR are assumed to work as for the other kinds. Probe T-1228
  checks them.
syntax.md points to the design. No code changes, eval unchanged.
