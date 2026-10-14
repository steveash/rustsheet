---
id: T-0424
title: "xlsx export: one helper formats theme colours and tints"
milestone: M11
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
One helper formats a theme colour (`theme` index plus a tint that is omitted when zero) for every xlsx export site.

## Context
T-0419 changed the tint format at three separate sites (crates/rustsheet-xlsx/src/export/styles.rs, styles_util.rs `get_color_xml`, dxfs_styles.rs); conditional_formatting.rs has two more variants. Reviewers noted the tint format can drift again.

## Acceptance criteria
- [ ] Theme colours are formatted in one place (`get_color_xml` or an extracted helper); no behavior change; existing export tests pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
