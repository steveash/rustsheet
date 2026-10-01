---
id: T-0104
title: "Python bindings on API v0"
milestone: M1
lane: api
status: todo
priority: P1
size: M
depends_on: [T-0103]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Expose the facade through PyO3 (`rustsheet.Workbook`, `open`, `edit` context manager, values, XlError values, typed exceptions), keeping the old Model bindings until parity.

## Context
bindings/python currently wraps Model/UserModel directly.

## Acceptance criteria
- [ ] New pytest suite for the facade API; GIL released around recalc/load/save.
- [ ] Docs/examples updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
numpy/arrow bulk (M12).
