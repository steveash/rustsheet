# T-1484: calcChain.xml and session edits vs WEEKNUM's stale state (excel-graph, 2026-10-19)

## Question

The probe cases (probes/weeknum-stale-*) answer the order Excel uses for a freshly loaded workbook.
Two questions do not fit a probe case, because the harness always uploads a fresh file, recalculates
it and reads it back:
1. Does a stored `calcChain.xml` change that order?
2. What happens to the state after formulas are entered or edited in a live session?

This was a one-off session against the excel-graph oracle, in the same way as the T-1853 research note
(docs/research/t1853-legacy-cell-edits-2026-10-18.md). It kept no sidecar, saved no file and
committed no script.

## Method

- **Workbooks.** Built with spec2xlsx on one sheet `S`, uploaded with `fullCalcOnLoad`, as the oracle
  does. The script used only `GraphBackend._drive` and `_call` from
  tools/src/rustsheet_tools/oracle/backends.py, with a non-persistent workbook session.
- **Markers and readers.** A *marker* is `=WEEKNUM(100,2)`, which sets Monday and shows 16, or
  `=WEEKNUM(100,1)`, which sets Sunday and shows 15. The *reader* is
  `=IFERROR(WEEKNUM(2,99)&WEEKNUM(3,99)&…&WEEKNUM(7,99),"-")`. It shows `222222` for a Monday state,
  `111111` for Sunday, and `-` when the state is empty.
- **Steps.** Edits are `PATCH range` with `formulas`. Each edit lets Excel recalculate the dirty
  cells. Explicit recalcs are `application/calculate` with `Recalculate`, `Full` or `FullRebuild`.
  The `values` of A1:A4 were read after every step.
- **calcChain.xml.** The xlsx was patched to add `xl/calcChain.xml`, listing the cells in a chosen
  order, with its content-type override and workbook relationship.

## Results

### calcChain.xml is ignored on a full-calc load

| case | cells | chain in the file | A1 after load, Full and FullRebuild |
|---|---|---|---|
| C0 | A1 reader, A2 Monday marker | none | `222222` |
| C1 | same | A1, A2 | `222222` |
| C2 | same | A2, A1 | `222222` |
| C3 | A1 reader, A2 Monday, A3 Sunday | A1, A2, A3 | `222222` |
| C4 | same | A3, A1, A2 | `222222` |
| C5 | same | A2, A1, A3 | `222222` |

C4 rules out the chain run forward, which would give `111111`. C5 rules out the chain run backward,
which would also give `111111`. Every case gives the layout's bottom-to-top value.

### Session edits

Column: the cell edited, then A1 (and A2 where it matters) after each step.

| case | loaded | edit | after the edit | Recalculate | Full | FullRebuild |
|---|---|---|---|---|---|---|
| E0 | A1 reader, A2 Monday: `222222` | none | | `222222` | `222222` | `222222` |
| E1 | A1 Monday | enter reader in A2 | A2 `-` | `-` | `-` | `-` |
| E2 | A1 reader, A2 Monday: `222222` | A2 := Sunday marker | A1 still `222222` (not recalculated) | `222222` | `-` | `-` |
| E3 | A2 Monday | enter reader in A1 | `-` | `-` | `-` | `-` |
| E4 | A1 reader, A2 Monday, A3 5 | A1 := reader reading A3; A3 := 6 | `-`, `-` | `-` | `-` | `-` |
| F2 | A1 reader, A2 Monday, A3 1 | A3 := 2 (a constant) | `222222` | | `222222` | `222222` |
| F3 | A1 reader, A2 Monday | re-enter A1 as is | `-` | | `-` | `-` |
| F4 | A1 reader, A2 Monday | re-enter A2 as is | `222222` | | `-` | `-` |
| F5 | A1 Monday, A2 reader: A2 `-` | A1 := Sunday marker | A2 `-` | | `-` | `-` |
| F6 | same as F5 | re-enter A2 as is | `-` | | `-` | `-` |
| F7 | A1 reader, A2 Monday | none (two Full recalcs) | | | `222222` | |
| F8 | A1 reader, A2 Monday, A3 Sunday | re-enter A3 as is | `222222` | | `222222` | `222222` |
| F9 | empty column | enter A2 Monday, then reader in A1 | `-` | | `-` | `222222` |
| F10 | empty column | enter reader in A1, then A2 Monday | `-` | | `222222` | `222222` |
| F11 | A1 and A2 readers, A3 Monday | re-enter A1 as is | `-`, A2 `222222` | | `-`, `222222` | `-`, `222222` |

### Reading

- **Recalculated alone.** A formula Excel recalculates on its own, after it is entered or because
  its input changed (E1, E3, E4, F3), sees an empty state. The state does not outlive a recalc. This
  agrees with probes/weeknum-stale-volatile-reader and probes/weeknum-stale-down-alone.
- **Hidden dependency.** Editing a marker does not recalculate the readers that used it (E2, F4).
  Excel does not track this dependency either.
- **Session formulas.** Once a formula is entered or re-entered in the session, it no longer shares
  the state with the formulas loaded from the file, in either direction and at every later full
  recalc, FullRebuild included (E2, F3, F4, F5, F6, F11). The loaded formulas still share it among
  themselves (F8, F11's A2). Formulas entered in the session share it with each other: F9 and F10 give
  `222222` at FullRebuild. Full, though, follows the order they were entered in: the latest-entered
  runs first (F9 `-`, F10 `222222`).
- **Edits that do not count.** Editing a constant changes nothing (F2), and neither does a plain
  recalc (F7).

None of this history is in the saved file. RustSheet recalculates the whole model on every
`evaluate()`. SPEC-0015 therefore matches the fresh-load order and leaves session history as a
non-goal.
