# 0132 — Unattributed visibly-off cells are listed by exit_owners.py from report.json

- **Status:** accepted (2026-10-27, T-0082)
- **Context:** Decision 0118 item 6 says Phase 1 cannot exit while a visibly-off Excel cell that is
  attributed to no function has neither an owning task nor a directive-5 `DEVIATION`. Nothing
  listed those cells. Whether an open task owns a cell is already decided in one place,
  `scripts/exit_owners.py` (decision 0096), and a second matcher in Rust would duplicate it
  (CLAUDE.md directive 3).
- **Decision:** `rustsheet-eval` writes every such cell to report.json `unattributed_off`
  (`case`, `cell`, `formula`). A `DEVIATION` cell scores as a deviation and is never listed.
  report.md prints the count on one line. `scripts/exit_owners.py --unattributed` (`make
  unowned-off`) drops the cells an open task of any milestone owns, using the same matching rule
  as `make exit-owners`, and prints the rest with a count. In the same change, report.json `run`
  gains `dirty` (`git status --porcelain` at the repository root; `null` without git).
- **Consequences:** The steward files tasks from `make unowned-off`. The list gates nothing:
  no CI job reads it, and `--strict` is for manual use. This change edits `scripts/` and the
  `Makefile`, which are protected paths; this record covers both.
