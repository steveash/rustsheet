---
id: T-0117
title: "Generator: operators-and-references policy for gen-m1 (offline, unfilled)"
milestone: M1
lane: harness
status: todo
priority: P1
size: M
depends_on: []
---

## Goal
`rustsheet-gen --policy operators-refs` writes a pending `gen-m1` suite of at least 3,000 formula cells that use only literals, operators and reference forms (no function calls), so T-0109 only needs the oracle fill.

## Context
M1 exit criteria require `gen-m1` (>= 3,000 oracle cells, operators and references only). The existing generator (`tools/src/rustsheet_tools/gen/generator.py`) is function-centric and `--policy numeric-rules` shows the pattern for a dedicated policy (`cli.py`). Coverage per `docs/milestones.md` M1: unary minus vs `^`, `%`, `&`, comparisons across types, text/number/bool coercion, error propagation, range/union/intersection operators, A1/`$`/mixed refs, whole rows and columns, cross-sheet and quoted sheet names. R1C1 parse is excluded from the generated formulas (xlsx stores A1).

## Acceptance criteria
- [ ] New policy generates deterministically from `--seed` (test: same seed gives byte-identical specs) and produces no function names (test scans every formula with a regex for `NAME(`; the union/parenthesised forms are allowed).
- [ ] Generated formulas cover each category above at least 50 times in a default run (test counts categories by construction tag, not by regex guess).
- [ ] Generation avoids volatile/locale-dependent constructs and keeps depth bounded; whole-column references only appear in operators that cannot create million-cell work (for example inside `:` or as an operand of intersection).
- [ ] Output suite `eval/suites/gen-m1` is NOT committed in this task (needs oracle fill, T-0109); CLI help documents the policy; `cd tools && uv run pytest` and `make check` green.

## Out of scope
Oracle fill and baselines (T-0109), function-based generation (T-0203).
