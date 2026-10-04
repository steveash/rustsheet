# 0020 — Merge conflicts are resolved at once, never waited out

- **Status:** accepted (2026-10-04, owner request; T-0028)
- **Context:** GitHub runs no `pull_request` CI on a PR that conflicts with its base. An agent that
  opened a PR and waits for the CI event therefore waits forever once `main` moves underneath it.
  On 2026-10-04 #56 and #53 both went `dirty` after #54 merged and sat idle until the owner asked.
- **Decision:** the `steward` skill makes mergeability the first check after every push, on every PR
  event or check-in, after any merge to `main`, and whenever a pushed head shows no check runs.
  A `dirty` PR is fixed immediately: merge `origin/main`, resolve, `make check`, push. This also
  covers stale agent PRs from other sessions (merge commit only). Agents arm a safety-net
  check-in after opening a PR so a lost event cannot strand it.
- **Also:** `make check` now runs `typos` (it was CI-only, so a clean local `make check` could
  still go red in CI; the session-start hook installs it). `make ci` keeps `deny` and `machete`.
- **Consequences:** more merge commits on agent branches (squash-merge hides them); no PR waits on
  CI that will never run. Routine prompts are unchanged: they already defer to these skills.
