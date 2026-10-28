---
id: T-2948
title: "Export writes the kept whitespace into the saved <f> text (SPEC-0023)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The xlsx export writes the whitespace the tree kept into the `<f>` text, as Excel saves it.

## Context
Excel saves typed whitespace in `<f>` (`=1 + 2` is `<f>1 + 2</f>`, `=SUM( 1 )`, `=NOW( )`); a line feed is raw CR LF bytes that read back as LF, so writing LF is equivalent (xlsx-storage.md "Typed whitespace is saved"; probes/t2786-whitespace-structured-refs-runs H14:H17). Our export prints a whitespace-free tree. Depends on SPEC-0023 step 1 (the parser keeping whitespace).

## Acceptance criteria
- [x] Export of a formula with kept whitespace writes it as typed; a stored `1 + 2` round-trips.
- [x] `make check` green; reviewers approve.

## Out of scope
- Whitespace inside structured references (T-2801) and beside missing arguments (T-2944).

## Outcome
Export prints each formula's written tree, so typed whitespace is saved in `<f>` (`=1 + 2` → `<f>1 + 2</f>`; a line feed
as `&#xA;`, which reads back as LF). The legacy `@` rewrite on import and export now decides on the tree with no
whitespace (`Node::rewrite_as_unspaced`): a shape site that does not peel `Spaced` costs the whitespace, never a value.
This also fixed import of an Excel `<f>` like `N( IF(1,{2;3}) )`, which took an extra `@`. A LET name written with spaces
is now prefixed `_xlpm.` on export. Eval delta: 0 (saved text only). Follow-ups: T-2970 (shape sites peel `Spaced`),
T-2971 (probe the derived saved shapes).
