---
id: T-3800
title: "Gate A: classify the 25 M9 engineering and database functions with no table or exemption, and file their table tasks"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Every in-scope numeric M9 engineering or database function that Gate A counts as missing ends up in one of two states:
- it has an `eval/accuracy/exempt.tsv` line backed by a decision record;
- or an open task names it in its title or Goal and will build its table.

## Context
- Gate A holds only with no missing table (SPEC-0029 "Gate A holds", decision 0188 R1(a)). Decision 0228 classified
  M5, M6 and M7. Its Consequences, and T-3551's Out of scope, leave M9's functions outside decision 0188's 18 for a
  separate classification task.
- `rustsheet-eval accuracy` on main (56917e82c) prints `missing tables 271`. 25 of them are M9 engineering (16) and
  database (9) functions that no Gate A task names:
  - engineering: BESSELI, BESSELJ, BESSELK, BESSELY, BIN2DEC, CONVERT, ERF, ERF.PRECISE, ERFC, ERFC.PRECISE, HEX2DEC,
    IMABS, IMAGINARY, IMARGUMENT, IMREAL, OCT2DEC;
  - database: DAVERAGE, DMAX, DMIN, DPRODUCT, DSTDEV, DSTDEVP, DSUM, DVAR, DVARP.
  Recount before filing, because tasks merged in the meantime may change the list.
- Questions that need judgment, each answered in the decision record:
  - **Database functions.** The D-functions take a database range and a criteria range (criteria.md "D-function front
    end across all twelve functions"), which literals cannot drive. They are `composite` unless a database argument
    accepts an array literal, naming the gen-m9 database cases and the D* probes. DCOUNT and DCOUNTA are already
    `exact`.
  - **Text-argument functions.** The generator's parameter kinds are `real`, `int`, `bool` and `array`
    (`accuracy/model.py` `Kind`), with no text. BIN2DEC, OCT2DEC and HEX2DEC turn digit text into a whole number:
    `exact` like ARABIC, or `composite` like DECIMAL (decision 0228). IMABS, IMARGUMENT, IMREAL and IMAGINARY take
    complex text, but IMABS and IMARGUMENT have floating-point kernels (engineering.md "IM* reciprocal kernels, IMABS
    and IMARGUMENT"). Decide between `composite` and a table that needs a text parameter kind. A text kind is a
    generator change, filed as its own task before the table task that needs it.
  - **CONVERT.** Its unit arguments are text from a fixed list, and its kernel is factor arithmetic (engineering.md
    "CONVERT constants and evaluation order", "CONVERT temperature"). A table needs the same text kind, or one
    Definition per unit pair drawn from a fixed list. Decide which, or `composite`.
  - **Bessel and error functions** need tables. Decision 0188 R3 puts Excel's BESSELJ about 1e-7 from the math,
    inside G = 1e-4, so Excel's value stays a valid target. Note the order cap and the underflow and overflow points
    (engineering.md "BESSEL order cap", "BESSELK underflow, BESSELI overflow and BIT shifts of 49 to 53") as the
    definitions' domain sources. ERF.PRECISE and ERFC.PRECISE share ERF's and ERFC's kernels. Apply T-3798's ruling
    on aliases if it has landed. Otherwise give each its own table.
- `exempt.tsv` is protected (decision 0199, T-3379): every line added needs the decision record. The three sibling
  classification tasks (T-3797, T-3798, T-3799) also add lines, so rebase before merging and keep the file sorted.
- Group the functions that need a table into table tasks of at most about seven functions each (M tasks, like
  T-3668 to T-3678): Bessel, error functions, and complex or CONVERT if they get tables. Create each with
  `make new-task MILESTONE=M0 LANE=harness`, `tier: opus`, `spec:` SPEC-0029.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) classifies each of the 25 functions (or the recounted list) as
      `exact`, `composite` or "needs a table (T-XXXX)", with a cited reason for each exemption.
- [ ] `eval/accuracy/exempt.tsv` gains the exempt lines, sorted. `rustsheet-eval accuracy` reports `missing tables`
      lower by their number, with no stale or bad exemption.
- [ ] Every function classified "needs a table" is named in the title or Goal of an open task filed by this PR. Any
      generator change those tables need (a text parameter kind) is its own task, and the table tasks depend on it.
- [ ] `python3 scripts/tasks.py validate` passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Building any table, definition or generator change (the filed tasks do that).
- The other M9 IM* functions, which return text and are not `returns: number` in the catalog.
- The other M9 families: financial (T-3797), distributions (T-3798), sample statistics (T-3799).
