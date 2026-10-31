---
id: T-3278
title: "rustsheet-gen --policy arg-pairs: draw calls that pass the argument-class pairs no Excel cell covers yet"
milestone: M2
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-3277]
---

## Goal
`rustsheet-gen --policy arg-pairs --milestone <M> [--functions ..]` writes cases whose calls each pass one
argument-class pair that report.json lists as unseen for that function (T-3277), with every forced
argument written so the eval classifier counts it as that class, so an oracle-filled batch raises the
function's `pairwise` share instead of hoping random draws hit the pair.

## Context
- Pairwise argument coverage (T-0083) averages 32.7% over 318 functions (docs/progress.md "Argument
  coverage", 2026-10-30). The M2 functions with two or more counted parameters sit at: MID 0.399, LEFT
  0.43, RIGHT 0.455, ROUNDDOWN 0.496, ROUNDUP 0.504, ROUND 0.521, INDEX 0.559, HLOOKUP 0.578, VLOOKUP
  0.619, MATCH 0.664, IF 0.675, IFERROR 0.744 (eval/runs/latest/report.json at 73b18722). Their single
  coverage is 100%, so each class reaches each parameter, but which of two arguments' errors wins, what
  an empty-text `num_chars` does beside a blank `text`, and so on, has no Excel cell. Argument-error
  precedence and coercion order are where parity bugs hide (for example dates.md "DATEDIF and DAYS360 error
  order with a bad unit or method").
- The classifier (crates/rustsheet-eval/src/argument_coverage.rs `observe`, `classify_value`) classifies
  a literal as written and a single-cell reference by the referenced value; nested calls and operators
  are not classified, so a forced argument must be a literal or a reference to a Data cell holding the
  class's value: blank is a reference to an empty cell, `range` a multi-cell reference, numeric text a
  quoted decimal (`"2.5"`), error a pooled error literal.
- Generator code: tools/src/rustsheet_tools/gen/generator.py (`argument`, `scalar_text`,
  `domain_value`; the policies in cli.py). Reuse the Data sheet layout and the catalog domains; a param
  with a `hard` domain is never forced outside it (Excel refuses the workbook, e.g. MUNIT), and such
  pairs are skipped and counted in the case description.
- The policy reads `argument_coverage[].unseen_pairs` from a report.json path (default
  eval/runs/latest/report.json); it never recomputes classes in Python (one definition per concept).
- Design choices this task settles (record them in docs/architecture/synthetic-generator.md): how other
  arguments of the call are drawn (plain in-domain values, so the forced pair decides the result), how
  many calls per pair (one is enough for coverage; two with different values per class reduce
  accidental agreement), how variadic parameters map (the classifier's `param_of`), and the
  determinism contract (same seed and report give the same cases).

## Acceptance criteria
- [ ] `rustsheet-gen --policy arg-pairs --milestone M2 --seed 0` writes cases under a gen suite whose
      formulas, once scored by `rustsheet-eval run` with the engine (no oracle needed for this check),
      cover at least 90% of the unseen pairs listed for the M2 functions in the input report; the
      skipped pairs are named in each case's description. Shown in the PR.
- [ ] tools/tests/test_generator.py: over a fixture report.json, every generated call passes its target
      pair (asserted by the forced argument texts and Data values), a hard-domain pair is skipped, and
      the output is deterministic for a seed.
- [ ] docs/architecture/synthetic-generator.md documents the policy and the design choices above.
- [ ] `make py-check` (ruff, mypy strict, pytest) green; `make check` green; reviewers approve.

## Out of scope
- Committing an oracle-filled batch (T-3279 for gen-m2; other milestones file their own).
- Changing the classifier or the pair definition (T-3277, T-0083).
- Pairs of three or more arguments.
