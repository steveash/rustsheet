---
id: T-0063
title: "rustsheet-gen --functions and --category narrow --focus to named functions or a catalog category"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
A sweep can target exactly the functions that need coverage. Today `--focus` cycles every function
of one milestone, so the 235 M9 functions below 400 Excel cells cannot be swept a category at a
time.

## Context
- `tools/src/rustsheet_tools/gen` (T-2108 added `--focus`): every formula's outer call is a
  function of exactly `--milestone`, cycled evenly.
- Decision 0094's sweeps (T-2252 to T-2258) need a narrower target:
  - all M9 statistical functions in one sweep, M9 financial in another;
  - a named list of the functions under 400 cells.

## Acceptance criteria
- [ ] `--category <name>` (a catalog `category`) and `--functions A,B,C` (catalog names) narrow
      `--focus` to those functions, still within `--milestone` and cycled evenly. Arguments stay
      in scope as today.
- [ ] Unknown names, an empty selection, or a function outside `--milestone` exit non-zero with
      a message.
- [ ] Tools tests: with a fixed seed, every selected function is the outer call of the same
      number of formulas (±1), and no other function is.
- [ ] `cd tools && uv run ruff check src tests && uv run mypy && uv run pytest -q` and
      `make check` green; reviewers approve.

## Out of scope
- New edge pools (T-2247 to T-2249 and T-2286).
- Generating or filling any suite.

## Visible-parity stage
Prerequisite for decision 0094's sweeps; its acceptance criteria are the whole stage.

## Outcome
`--category` and `--functions` (comma-separated catalog names) narrow `--focus` in `rustsheet-gen`
(`narrow_targets` in `gen/generator.py`). Unknown names or categories, functions outside
`--milestone` or not generator-ready, an empty selection, and use without `--focus` exit non-zero
with a message. Tools tests cover even cycling (exactly even within one case) and each error.
Eval delta: none (tools only).
