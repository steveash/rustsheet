# 0008 — Routine prompts are versioned in the repo

- **Status:** accepted (2026-10-01)
- **Context:** The autonomous loop runs as scheduled Claude Code routines. Routines created from a
  session via the remote-control tool carry no repository source or GitHub tools, so their
  sessions cannot check out the repo or open/merge PRs.
- **Decision:** Routines are created by the owner in the claude.ai routines UI with the repo and
  GitHub attached. Their prompts and schedules live in `docs/process/routines.md`. Changing the
  loop's behavior means changing that file (protected path) and the routine together.
- Agents never create, modify or delete routines (enforced by a permission deny in
  `.claude/settings.json`); only the owner does, keeping the 2-worker cap of decision 0005.
- The work-loop skill records the `docs/status.md` line on the task branch before merging (agents
  cannot push to `main`).
- The `docs/status.md` "Critical lane" line is an ordered comma-separated lane list; workers accept
  only allow-listed lane names.
- **Consequences:** The loop's instructions are reviewable and versioned like code. Agent-workflow
  points to routines.md instead of restating the prompts.
