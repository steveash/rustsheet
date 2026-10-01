# 0008 — Routine prompts are versioned in the repo

- **Status:** accepted (2026-10-01)
- **Context:** The autonomous loop runs as scheduled Claude Code routines. Routines created from a
  session via the remote-control tool carry no repository source or GitHub tools, so their
  sessions cannot check out the repo or open/merge PRs.
- **Decision:** Routines are created by the owner in the claude.ai routines UI with the repo and
  GitHub attached. Their prompts and schedules live in `docs/process/routines.md`. Changing the
  loop's behavior means changing that file (protected path) and the routine together.
- **Consequences:** The loop's instructions are reviewable and versioned like code.
