---
id: T-1016
title: "Desktop probe: typed #BLOCKED!/#GETTING_DATA cell input (SPEC-0003)"
milestone: M1
lane: research
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: []
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
Evidence of what desktop Excel stores when a user types `#BLOCKED!`, `#CONNECT!`, `#FIELD!` or `#GETTING_DATA` (any case) into a cell: an error constant, or text.

## Context
SPEC-0003 "Two name tables" chose, without evidence, that typed `#GETTING_DATA` becomes an error and typed `#BLOCKED!` stays text (syntax.md "Newer error literals", UNVERIFIED). Typed input goes through the UI, which the excel-graph oracle cannot drive. It needs the `excel-desktop` backend (docs/architecture/oracle.md), for example COM `Range.Value = "#BLOCKED!"` followed by `ERROR.TYPE`/`ISTEXT` next to it.

## Acceptance criteria
- [ ] Each typed name's result is recorded in syntax.md with desktop evidence, and SPEC-0003 and `Model::set_user_input` are updated if they differ.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Formula literals (T-0175, done) and stored constants (T-0999).

## Blocked because
It needs the `excel-desktop` oracle backend, which is `n/a` in cloud sessions (`cd tools && uv run rustsheet-oracle status` shows `excel-desktop  n/a`). Unblock when that line reads OK.
