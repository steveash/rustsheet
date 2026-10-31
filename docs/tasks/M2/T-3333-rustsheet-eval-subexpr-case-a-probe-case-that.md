---
id: T-3333
title: "rustsheet-eval subexpr-case: a probe case that asks Excel for every subexpression of a visibly-off spec-case cell, and show the first one that diverges"
milestone: M2
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
An agent with a visibly-off cell whose formula nests several calls can find the call that goes
wrong with one oracle fill. It runs `rustsheet-eval subexpr-case <case> <Sheet!A1>`, which writes
a probe case holding each subexpression of that formula in its own cell over the same input
cells. It fills that case with `rustsheet-oracle fill`, then runs `rustsheet-eval subexpr-case
--culprit <probe case>`. That prints the innermost subexpression whose value is visibly off while
all of its children match Excel.

## Context
- The report holds 935 visibly-off Excel cells that decision 0118 attributes to no function
  (report.json `unattributed_off`, full run at 9453cfb5 on 2026-10-31). 790 are in gen-m9 and 117
  in probes. Each one's formula nests several calls, for example
  `=CHISQ.DIST(T.INV.2T(0.975,7),6,Data!E12)` or `=HYPGEOM.DIST(IRR(...),3,2,19,NOT(...))`.
  Today an owner finds the culprit by hand. It writes a probe of the inner calls, as T-2940 does
  for its 38 cells, or it guesses from the engine's own subexpression values. The guess fails
  when two calls are each a little off.
- The design is docs/architecture/synthetic-generator.md §11 stage 1 ("oracle-free
  localization") with §9.3's subexpression probing. One oracle call evaluates every node.
  Nothing implements it yet, and no task covers it (no `reducer`, `localize` or `subexpression`
  task is open).
- Reuse the engine parser through a `#[doc(hidden)]` testing hook next to `call_arguments`
  (crates/rustsheet-engine/src/call_arguments.rs, T-0077). The harness must not get a second
  formula parser (CLAUDE.md directive 3). Print nodes with the engine's own printer
  (expressions/parser/stringify.rs).
- Context traps the case must handle, or skip with a note in the case description:
  - Relative references must still point at the original cells once the text sits in another
    cell. Write the subexpression at the original cell's position on a copy of its sheet, or
    print its references absolute.
  - A node that yields a multi-cell range or array spills on its own. Under the original call it
    may have been implicitly intersected or aggregated. Check only scalar-context nodes, or wrap
    nothing and record that the node spills.
  - Nodes that read LET or LAMBDA parameters cannot leave their scope. Volatile nodes (RAND,
    NOW) cannot be compared. Skip both.
- Milestone: decision 0051 assigns an untagged probe to M2. A localization case of a gen-m9
  cell must not land in the M0-M2 exit row, so the case carries its source case's milestone tag.

## Acceptance criteria
- [ ] `rustsheet-eval subexpr-case <suite/case> <Sheet!A1> [--out PATH]` writes a valid
      `rustsheet-case/v1` spec case under `eval/suites/probes/cases/` named
      `subexpr-<source stem>-<cell>`. It holds the source case's input cells, one cell per distinct
      non-literal subexpression in post-order (children first), the full formula as its last
      cell, a `check` list of those cells and the source case's milestone tag. Its description
      names the source case and cell and maps each cell to its subexpression text.
- [ ] It refuses an xlsx-backed case (calipers, poi, ironcalc-calc) with a clear error and no
      panic. It also refuses a cell that holds no formula.
- [ ] `--culprit <probe case>` reads the case's oracle sidecar and the engine's values. It prints
      the innermost visibly-off node whose children all match (decision 0083's rule, `closeness.rs`;
      no second rule), or `no divergent node`.
- [ ] Rust tests cover: relative and absolute same-sheet references keep their targets; LET/LAMBDA
      and volatile nodes are skipped; a hand-written oracle sidecar where only an inner call is off
      gives that call as the culprit; a sidecar where everything matches gives `no divergent node`.
- [ ] Demonstrated on one gen-m9 cell from report.json `unattributed_off`. The case is filled with
      excel-graph and committed, and `--culprit` output is pasted in the task Outcome. If the
      culprit is not already owned, add it to the owning task or file a follow-up.
- [ ] docs/architecture/eval-harness.md "CLI" documents the command. `make check` green; reviewers approve.

## Out of scope
- Workbook ddmin, AST hoisting and literal shrinking (§11 stages 2-5). File them as follow-ups if
  this stage proves useful.
- Running it over all 935 unattributed cells, or re-owning them. That is the owners' and the
  gap-triager's work.
- xlsx-backed cases (no spec to copy the inputs from).
