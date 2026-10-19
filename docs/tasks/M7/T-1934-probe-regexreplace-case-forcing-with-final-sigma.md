---
id: T-1934
title: "Probe REGEXREPLACE case forcing with final sigma"
milestone: M7
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
REGEXREPLACE case forcing lowers a capital sigma to the final or medial form exactly as Excel does.

## Context
T-1897 case-forces each replacement piece separately (`CaseState::push`), so LOWER's final-sigma lookahead never sees across pieces or past a `\u`/`\l` head character. Probe `\lΣa`, `\L$1x` with `$1`=ΑΣ, `\LΑΣ b`, then fix `CaseState` if Excel differs.

## Acceptance criteria
- [ ] A probe case with oracle values covers the sigma cases; text.md records them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
