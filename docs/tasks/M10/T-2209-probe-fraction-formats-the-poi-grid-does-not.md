---
id: T-2209
title: "Probe fraction formats the poi grid does not cover: mixed 0/? denominators, spaces or quoted text around the slash"
milestone: M10
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's text is known for the fraction shapes T-0982 implemented by extrapolation, and the engine matches it or has a task to.

## Context
T-0982 fitted the fraction rules to poi/NumberFormatTests A120:A332 (number-formats.md "Fraction formats", "Literals between the parts"). Reviewers noted shapes that grid does not cover; the engine's current choice for each is listed there as unprobed:
- mixed denominators `# ?/0?` and `# ?/?0` (zeros are padded on the left, spaces on the right);
- plain spaces or quoted text around the slash (`0 / 0`, `# "x"/4`, `# ?/"y"4`), which now make a fraction;
- a `/` followed by literals but no denominator (`0 /"x"`), which stays a literal;
- integer kinds and literal mixes beyond `|I\:N=/=D|`, e.g. `#? \:?#=/=0?`.
Use the `excel-probe` skill with values 0, 0.5, 1.25, -0.75, 23.75.

## Acceptance criteria
- [ ] A probe case with the shapes above has oracle values; number-formats.md records the findings.
- [ ] Mismatches are fixed here when they are small, or filed as tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-2072, T-2074.
