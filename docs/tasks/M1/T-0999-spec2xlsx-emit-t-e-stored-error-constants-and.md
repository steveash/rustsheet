---
id: T-0999
title: "spec2xlsx: emit t=e stored error constants and probe #BLOCKED! etc."
milestone: M1
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
spec2xlsx can write an error cell as a stored `<c t="e"><v>#BLOCKED!</v></c>` constant, and the probes suite records whether Excel loads stored `#BLOCKED!`, `#CONNECT!`, `#FIELD!`, `#BUSY!`, `#UNKNOWN!`.

## Context
Follow-up of T-0195 / T-0175. `tools/src/rustsheet_tools/spec2xlsx.py` (the `tag == "e"` branch) writes a formula `=#X`, and Excel for the web rejects those names as formula grammar, so stored error constants are unprobed. xlsxwriter has no stored-error API, so the cell XML must be patched after the write. Record the result in docs/excel-semantics/syntax.md ("Newer error literals").

## Acceptance criteria
- [ ] A typed `{"t":"e"}` cell for a name Excel accepts as a formula literal still round-trips; a new opt-in form emits `t="e"` constants.
- [ ] A probe case plus excel-graph sidecar answers whether each stored name loads; findings in syntax.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine support for the new error kinds (T-0177).

## Outcome
spec2xlsx writes `{"t":"e","v":..,"stored":true}` as a stored `t="e"` constant (placeholder cell patched after write; deterministic; tests in tools/tests/test_spec2xlsx_stored_errors.py). Probe probes/error-stored-constants: stored `#BUSY!` loads, `#BLOCKED!`, `#CONNECT!`, `#FIELD!`, `#UNKNOWN!`, `#CALC!` are rejected; findings in syntax.md. Eval: probes new-coverage failures +6 (engine does not support the new error kinds, T-0177), no regressions.
