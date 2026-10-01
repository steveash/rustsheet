# 0005 — Autonomous merges without human review

- **Status:** accepted (2026-10-01), per owner instruction.
- **Decision:** A PR merges (squash) when CI is green and every applicable reviewer agent
  (security, architecture, quality, Excel fidelity, performance) approves. No human approval is
  required. Governance is enforced mechanically:
  - ratchets (eval baseline, lint debt, duplication, file size)
  - `scripts/check_governance.py`, which requires decision records for protected-path changes
  - permission denies for force-pushes and direct pushes to `main` in `.claude/settings.json`
- **Consequences:** At most 2 concurrent workers until the revert rate is proven low. A weekly
  spot-audit of merged PRs checks for reviewer misses.
