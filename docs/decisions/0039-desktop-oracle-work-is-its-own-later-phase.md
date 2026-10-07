# 0039 — Desktop-oracle work is its own later phase (Phase 4)

- **Status:** accepted (2026-10-07, owner request)
- **Context:**
  - The owner (2026-10-07): "I have no plan to set up a desktop oracle any time soon. So track
    the items but don't block me or anything on needing a desktop oracle. That will be a whole
    new phase."
  - The `excel-desktop` backend exists in code (`tools/src/rustsheet_tools/oracle/backends.py`,
    COM via pywin32), but it needs an owner-run Windows machine with Microsoft 365 Apps and a
    self-hosted runner (docs/architecture/oracle.md, "Owner setup — Windows desktop backend").
    `rustsheet-oracle status` shows `excel-desktop  n/a` in every session so far.
  - Several open tasks waited on it. Some cannot be done at all on Excel for the web: T-1016
    (typed cell input), T-1051 (conditional-format evaluation), T-0171 (fill handle) and T-0198
    (structural edits to 3-D spans). Others had a "confirm on desktop first" gate although
    excel-graph evidence already exists: T-0124, T-0221, T-0934 and T-0941. docs/status.md
    listed the desktop oracle under "Needs owner", and decision 0019 and
    docs/excel-semantics/README.md said nothing is implemented until an excel-desktop probe
    decides.
- **Decision:**
  1. **Phase 4, "Desktop oracle".** Work that needs evidence only desktop Excel can give goes to
     milestone M14 ("Desktop-oracle parity") in a new Phase 4, ordered after Phase 3 (Release).
     Phase 4 is not started. Examples: desktop-only behavior (`CELL("width")`, workbooks Excel
     for the web cannot open), conditional-format evaluation, typed cell input, fill-handle
     behavior, structural-edit semantics, and settling web-vs-desktop conflicts. Its tasks keep
     their ids, move to `docs/tasks/M14/` and stay `status: blocked` until
     `rustsheet-oracle status` shows `excel-desktop OK`.
  2. **excel-graph is the oracle of record for Phases 1–3.** Its values are goldens and
     evidence. A task that has excel-graph evidence implements on it. It does not wait for a
     desktop confirmation. The confirmation becomes an acceptance item of an M14 task (T-1075).
  3. **Desktop-only questions never block.** If a failure's only open question needs desktop
     evidence, its M14 task counts as "a filed task" for the Phase 1 exit and for every
     milestone exit. It never blocks a milestone or phase exit, the steward, or workers. The
     desktop oracle is not a "Needs owner" item. Setting the oracle up is how the owner starts
     Phase 4.
  4. **Web-vs-desktop conflicts: match the web for now.** This keeps the repo's current handling
     of poi conflicts:
     - poi is scored against excel-graph sidecars, which replace the cached desktop values
       (`prepare_xlsx`, decision 0030 "Sidecars, today").
     - A cell where the web materially contradicts a cached desktop value is left pending by
       `rustsheet-oracle fill` (`CONFLICT`). It is unscored, so it is neither a pass nor a
       failure (decision 0019; 257 cells in T-0019, e.g. lowercase-`e` formats and an elapsed
       token after a plain `s`).
     - Below 1e-9 relative, the fill writes the 16.x desktop number (T-0142).
     A task that must choose a value for a CONFLICTING or UNVERIFIED-pending-desktop behavior
     targets the excel-graph value and names the M14 task that will confirm it. For Phases 1–3
     this replaces three rules: decision 0030 rule 1 ("desktop decides"), the exception in 0030
     rule 3 (web values are never evidence for a known divergence), and decision 0019's "nothing
     is implemented until an excel-desktop probe decides". These stay as they are:
     - desktop evidence already in the repo that a cell passes against today (ironcalc-calc
       cached values, T-0142's 16.x last-digit values, the `INDIRECT` missing-sheet cells).
       Moving such a cell to the web value would be a regression (CLAUDE.md directive 2) and
       needs its own decision;
     - decision 0030 rules 2 and 4 (non-en-US and pre-16 desktop evidence, last-digit fitting
       targets);
     - decision 0019's pending mechanism in the fill.
- **Consequences:**
  - docs/milestones.md gains the Phase 4 row and M14. The tasks that are entirely
    desktop-dependent (T-1016, T-1051, T-0171, T-0198) move to M14 as `blocked` and depend on
    T-1074, the backend bring-up and calibration. T-0124, T-0221, T-0934 and T-0941 stay where
    they are, with their desktop gates removed. Their confirmations are items of T-1075.
  - The steward drops the "excel-desktop oracle" entry from "Needs owner" in docs/status.md at
    its next run and does not add it back. docs/process/agent-workflow.md ("Missing capability")
    and the `excel-probe` skill say that a desktop-only need goes to M14 rather than blocking
    or going to the owner.
  - docs/excel-semantics/README.md and the CONFLICTING entries in number-formats.md cite this
    record. New desktop-only questions get an M14 task (or an item on T-1075) in the PR that
    finds them.
  - Decision 0030 rule 1 and decision 0019's implementation gate apply again when Phase 4
    starts. That is when the desktop backend's values can replace web goldens. Each such change
    is its own PR with the evidence per cell (decision 0030 "Sidecars, to do").
