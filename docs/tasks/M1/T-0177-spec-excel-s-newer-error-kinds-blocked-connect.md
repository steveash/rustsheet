---
id: T-0177
title: "Spec: Excel's newer error kinds (#BLOCKED!, #CONNECT!, #FIELD!, ...) in engine, importer, facade ErrorKind and Python"
milestone: M1
lane: api
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A merged spec (next free `docs/specs/SPEC-NNNN-*.md` number; SPEC-0002 is reserved for the
dependency graph) that decides how RustSheet represents Excel's newer error values (`#BLOCKED!`,
`#CONNECT!`, `#FIELD!`, and whichever of `#BUSY!`, `#GETTING_DATA`, `#UNKNOWN!`, `#PYTHON!`,
`#EXTERNAL!` the evidence supports) end to end, and that lists the implementation tasks, so the 13
failing probes/rich-errors-* cells and T-0124's `#BLOCKED!` can be fixed without inventing the
design inside an engine PR.

## Context
This changes the public API (`rustsheet::ErrorKind`, `#[non_exhaustive]`; Python `XlError`), the
engine's `Error` enum and literal table (crates/rustsheet-engine/src/expressions/token.rs, also the
single ERROR.TYPE table `Error::error_type_code`), and the importer (`value_metadata_errors` in
crates/rustsheet-xlsx/src/import/rich_errors.rs, which today leaves errors the engine has no kind for
as `#VALUE!`), so CLAUDE.md directive 4 requires a spec first. Evidence available:
docs/excel-semantics/arrays.md (errorType → name → ERROR.TYPE table, T-0030), references.md
(external reference with no cached data is `#BLOCKED!`), probes/rich-errors-field/-image/-stockhistory
(13 known failures in eval/baseline/probes.txt). Decision 0027 keeps IMAGE, STOCKHISTORY and linked
data types out of scope for their values; the spec decides what those functions evaluate to in a
headless engine with no I/O (directive 5), for example the error Excel gives when the service is
unreachable, and records any difference as a DEVIATION. T-0175 (research) probes whether the
literals parse in formula text; if it is not done when this spec is written, the spec states the
parsing question as open and makes the parsing task depend on T-0175.

The spec covers: the variants and their literals; ERROR.TYPE codes; parsing and printing; xlsx import
(rich-value `vm` errors and plain `t="e"` cells) and export (what we write so Excel reads it back);
error precedence where they meet other errors; the facade `ErrorKind`/`code()` mapping and the
`literals_match_engine` sync test; Python exposure; what the out-of-scope functions return.

## Acceptance criteria
- [ ] `docs/specs/SPEC-NNNN-*.md` from docs/specs/TEMPLATE.md, covering every point above, each
      behavior citing its docs/excel-semantics entry or marked as needing evidence (with the task
      that gets it).
- [ ] A rollout table of S/M implementation tasks (lane, size, dependencies); the tasks are filed
      with `make new-task` and link the spec in `spec:`.
- [ ] Architecture and Excel-fidelity reviewers approve; `make check` green.

## Out of scope
- Any code change (the implementation tasks do it).
- Values of IMAGE, STOCKHISTORY, FIELDVALUE beyond their error (decision 0027).
