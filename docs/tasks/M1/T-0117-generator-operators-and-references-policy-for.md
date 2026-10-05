---
id: T-0117
title: "Generator: operators-and-references policy for gen-m1 (offline, unfilled)"
milestone: M1
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
`rustsheet-gen --policy operators-refs` writes a pending `gen-m1` suite of at least 3,000 formula cells that use only literals, operators and reference forms (no function calls), so T-0109 only needs the oracle fill.

## Context
M1 exit criteria require `gen-m1` (>= 3,000 oracle cells, operators and references only). The existing generator (`tools/src/rustsheet_tools/gen/generator.py`) is function-centric and `--policy numeric-rules` shows the pattern for a dedicated policy (`cli.py`). Coverage per `docs/milestones.md` M1: unary minus vs `^`, `%`, `&`, comparisons across types, text/number/bool coercion, error propagation, range/union/intersection operators, A1/`$`/mixed refs, whole rows and columns, cross-sheet and quoted sheet names. R1C1 parse is excluded from the generated formulas (xlsx stores A1).

## Acceptance criteria
- [x] New policy generates deterministically from `--seed` (test: same seed gives byte-identical specs) and produces no function names (test scans every formula with a regex for `NAME(`; the union/parenthesised forms are allowed).
- [x] Generated formulas cover each category above at least 50 times in a default run (test counts categories by construction tag, not by regex guess).
- [x] Generation avoids volatile/locale-dependent constructs and keeps depth bounded; whole-column references only appear in operators that cannot create million-cell work (for example inside `:` or as an operand of intersection).
- [x] Output suite `eval/suites/gen-m1` is NOT committed in this task (needs oracle fill, T-0109); CLI help documents the policy; `cd tools && uv run pytest` and `make check` green.

## Out of scope
Oracle fill and baselines (T-0109), function-based generation (T-0203).

## Outcome
`rustsheet-gen --policy operators-refs [--seed N]` (`tools/src/rustsheet_tools/gen/operators_refs.py`)
writes suite `gen-m1`. It makes one case per category, 15 categories (literal, unary-power,
percent, concat, compare, coerce, error, refs, range, intersection, union, whole-row-col,
cross-sheet, chain, mixed). Each has 250 probes, all distinct within the case, so the default
seed gives 3,750 cells. Every category is a table of `form: template` lines, and each probe is
tagged `category/form` in `hypotheses`.

The data lives on sheets `Data`, `'My Data'` and `'Q''1'`. Every intersection covers one cell or
none by construction, so nothing spills into the next probe. Whole rows and columns appear only as
intersection operands, and chains reference earlier probes only.

Tests in `tools/tests/test_generator.py` cover:
- determinism;
- no function names;
- every category at least 50 times, with at least 3 forms and at least 3,000 distinct probes;
- the whole-row/column rule;
- no spills, with intersection cell counts computed from the areas;
- nesting depth at most 6;
- no date, currency or separator text;
- acyclic chains.

A local crash check (`uv run rustsheet-gen --policy operators-refs`, then
`rustsheet-eval run --suite gen-m1`, then deleting the suite) gave 15 cases pending and 0 crashes.
The suite itself is not committed: T-0109 fills it with the oracle, after first smoke-probing one
formula per form tag in a single workbook.
