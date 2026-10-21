---
id: T-2214
title: "Remove unused GIT_VERSION build script output after INFO became #VALUE!"
milestone: M5
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine no longer emits an unused `GIT_VERSION` env var.

## Context
From T-2190 review: INFO("release") was the only consumer of `GIT_VERSION` (crates/rustsheet-engine/build.rs). `get_all_timezones` stays (public export).

## Acceptance criteria
- [ ] build.rs and its env output removed (or justified if another consumer exists); `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
