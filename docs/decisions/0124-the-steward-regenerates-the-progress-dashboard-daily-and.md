# 0124 — The steward regenerates the progress dashboard daily and status.md drops its scoreboard

- **Status:** accepted (2026-10-26, T-0076; SPEC-0022; owner decision 2026-10-25 in session
  https://claude.ai/code/session_014ASuNyVecGvKjcZUDfWLKA, SPEC-0022 "Owner decisions" item 2).
- **Amends:** decision 0094's process part (the copy of the checkpoint line into docs/status.md),
  as decision 0118 announced; decision 0114 item 3's trigger for the Phase 1 exit line. Decision
  0008 is unchanged: docs/process/routines.md stays the routines' single definition, and only the
  owner changes the live routine.
- **Context:**
  - `make progress-report` (decision 0120) and the `progress-report` skill (T-0075, decision 0123)
    produce SPEC-0022's dashboard, docs/progress.md, but no routine runs them. The steward's
    prompt still told it to keep a per-suite scoreboard in docs/status.md, so the two pages would
    carry the same numbers twice, and docs/status.md has no trend or tiers.
  - The steward skill's "Fifteen-digit parity checkpoint" rule copied the checkpoint and Phase 1
    lines into docs/status.md and answered "was the previous run also met" from the previous
    snapshot. docs/progress/history.json (T-0071) now keeps one row per run with `v15.met`.
  - The `**Phase 1:** exited` rule read a Phase 1 line in docs/status.md, which leaves the
    snapshot.
  - The protected paths changed here: `.claude/skills/steward/SKILL.md` and
    `docs/process/routines.md` (scripts/check_governance.py).
- **Decision:**
  1. **Routine prompt.** The steward block of docs/process/routines.md is replaced, byte for
     byte, with T-0076's "New steward prompt". Step (2) runs `make progress-report` after
     `make eval` and `make eval-lower-visible-lock`, even on a failed eval or fetch, then the
     `progress-report` skill until `rustsheet-progress-report --check` passes; the steward PR
     carries docs/progress.md and docs/progress/. Step (1) keeps only the health section in
     docs/status.md. The old steps (2)–(6) become (3)–(7), unchanged.
  2. **Steward skill.** A new "Progress dashboard" rule states the order (`make eval`,
     `make eval-lower-visible-lock`, `make progress-report`, the `progress-report` skill) and
     points to SPEC-0022's failure handling. The checkpoint rule no longer copies lines into
     docs/status.md and reads the previous run from the last complete history.json row, citing
     decision 0118 for the Phase 1 measure. The Phase 1 exit rule reads
     `v15_checkpoint.phase1_met` from eval/runs/latest/report.json (shown in the dashboard's
     `goals` block).
  3. **docs/status.md.** Its first line under the intro links the dashboard. The per-suite table,
     the fifteen-digit and Phase 1 lines and the milestone-exit measure lines leave the snapshot;
     SPEC-0022's "Kept" items stay.
- **Consequences:**
  - The live "rustsheet steward (daily)" routine changes only when the owner pastes the prompt;
    until then the steward runs the old prompt, and its skill already asks for the dashboard step.
  - Per-suite, parity, Phase 1 and milestone-exit numbers have one home, docs/progress.md, with
    history and deltas. docs/status.md stays the operational log.
  - No gate, baseline or CI job changes. Reviewers `reviewer-architecture` and `reviewer-security`
    approve the protected paths explicitly (T-0076).
