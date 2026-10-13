---
id: T-1504
title: "Give the #NAME? of calling a number or boolean variable a message"
milestone: M1
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Calling a LET variable or LAMBDA parameter that holds a number or boolean gives `#NAME?` with a message.

## Context
Split from T-1223. `called_value` in functions/logical/lambda.rs builds `ArrayNode::Error(Error::NAME)`, which has no message, so the cell error has an empty one (e.g. "A value cannot be called"). Messages are not Excel-visible; keep values identical.

## Acceptance criteria
- [ ] A unit test pins the message; eval shows 0 changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
