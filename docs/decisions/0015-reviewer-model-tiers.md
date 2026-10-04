# 0015 — Reviewer agents run on model tiers

- **Status:** accepted (2026-10-04, owner request; T-0022)
- **Context:**
  - Every PR runs 3-5 reviewer agents, each re-reading the diff and the code around it
    (20k-70k tokens each on 2026-10-04). Usage limits, not wall-clock, bound throughput.
  - Per-MTok prices: Sonnet 5.5 $2 in / $10 out, Opus 5.5 $4 / $20.
  - The backlog PR (#49) was reviewed by security, architecture and quality on Sonnet at ~21k
    tokens each; their findings were specific and correct.
- **Decision:**
  - `reviewer-security`, `reviewer-architecture`, `reviewer-quality`: `model: sonnet`. Their
    checks are mostly rule-based (layering, duplication, lint, panic paths, secrets, CI abuse).
  - `reviewer-excel-fidelity`, `reviewer-performance`: `model: opus`, pinned explicitly so a
    Sonnet worker (T-0023) does not downgrade them. Numeric parity, oracle evidence and
    complexity arguments need the stronger model.
  - The `review` skill uses the frontmatter model and never overrides it per call.
- **Consequences:**
  - Review cost per PR drops roughly by half for the three routine reviewers; no gate changes:
    every applicable reviewer must still APPROVE.
  - Check after one week: the steward's spot-audit (routines step 6) records, per reviewer, any
    defect a re-review finds that the original review missed. A miss attributable to the cheaper
    model reverts that reviewer to `model: opus` via a follow-up decision.
