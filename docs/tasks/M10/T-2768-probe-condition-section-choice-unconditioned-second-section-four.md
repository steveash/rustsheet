---
id: T-2768
title: "Probe condition section choice: unconditioned second section, four sections, <=0 / =-1 / <0 sign cuts"
milestone: M10
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's section choice is known for a three-section format whose second section has no condition, a four-section format, and the sign cut for `[<=0]`, `[=-1]` and `[<0]` with digits.

## Context
T-1734 implemented the choice from gen-m10 evidence; these paths are unprobed (number-formats.md, Conditional sections). Code: `conditional_section` in formatter/parser.rs; with an unconditioned second section it falls back to the sign rules.

## Acceptance criteria
- [ ] A probes case with those formats, oracle-filled; number-formats.md states each result; fix filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
