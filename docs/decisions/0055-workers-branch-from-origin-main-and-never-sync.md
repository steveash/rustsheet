# 0055 — Workers branch from origin/main and never sync a local main

- **Status:** accepted (2026-10-14, owner request, T-0055).
- **Context:** The sonnet B run at 05:46 UTC on 2026-10-14 did no work. Its container's local
  `main` was a stale shallow checkout (last commit #440), so the routine's `git checkout main &&
  git pull --ff-only` found no common ancestor with `origin/main`. The run then ran `git reset
  --hard origin/main`, which the auto-mode classifier treats as irreversible local destruction;
  it denied the run's next (read-only) commands and the run stopped. `origin/main` was never
  rewritten. The work-loop skill step 1, the three routine prompts in `docs/process/routines.md`
  and the agent-workflow loop summary all synced a local `main`.
- **Decision:** No run checks out, pulls into or resets a local `main`. Every task branch is cut
  directly from a freshly fetched `origin/main` (`git fetch origin main && git checkout -B
  agent/<lane>/T-XXXX-<slug> origin/main`); between tasks a run fetches `origin/main` again and
  branches from it. Divergence is never repaired with `git reset --hard` or by deleting a branch.
  When decision 0028's `git merge-base --is-ancestor origin/main origin/<branch>` check fails in a
  shallow clone, the remedy is `git fetch --unshallow origin` (or a deeper fetch), not a reset.
- **Consequences:** The work-loop and steward skills, `agent-workflow.md` and the three prompts in
  `routines.md` say this. The owner must re-apply the three routine prompts (sonnet worker, opus
  worker, steward) from `routines.md`, since agents never modify routines; until then the old
  prompts still say "sync `main`", and the skill's step 1 tells a run to fetch and branch instead.
  The existing `deny` rule for `git reset --hard origin/main` in `.claude/settings.json` stays.
