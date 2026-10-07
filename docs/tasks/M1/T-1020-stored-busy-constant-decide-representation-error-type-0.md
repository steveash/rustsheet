---
id: T-1020
title: "Stored #BUSY! constant: decide representation (ERROR.TYPE 0) per SPEC-0003"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1010]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
Importing a workbook that stores `<c t="e"><v>#BUSY!</v></c>` gives Excel's value. Today it becomes `#ERROR!` because the engine has no `#BUSY!` kind.

## Context
T-0999 found that Excel loads a stored `#BUSY!` as an error that is not `#N/A`, with `ERROR.TYPE` 0, `TYPE` 16 and `ISERR` TRUE (docs/excel-semantics/syntax.md, "Newer error literals"). SPEC-0003 left `#BUSY!` as a non-goal because nothing produces it. An `ERROR.TYPE` of 0 does not fit `Error::error_type_code` as it stands (a `u8` table with no 0). Decide, by amending SPEC-0003, whether it becomes a fifth kind, and how `code()` and the facade represent it.

## Acceptance criteria
- [ ] SPEC-0003 is amended with the decision, citing T-0999's probe cells.
- [ ] If the decision is to add the kind: it is implemented, with the T-0999 probe cells passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `#PYTHON!` and `#EXTERNAL!` (unprobed).
