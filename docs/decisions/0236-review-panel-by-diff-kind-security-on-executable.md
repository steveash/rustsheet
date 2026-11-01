# 0236 — Review panel by diff kind: security on executable changes, one design reviewer

- **Status:** accepted (2026-11-01, owner request after a review-cost investigation, T-3702). It
  amends 0015 (reviewer tiers) and the panel of 0081 and 0178.
- **Context:**
  - The "Agent review" tables of 271 of the last 300 merged PRs (2026-11-01) were parsed. A
    reviewer counts as having found something when it requested changes, or a later round fixed
    its finding; the method is heuristic.
    - excel-fidelity: about 52 of about 190 applicable PRs (27%).
    - quality: about 32 of 265 (12%), mostly duplicated helpers.
    - architecture: about 20 (7%), about half of them task-file or process wording.
    - security: about 8 (3%). About 4 were substantive, and all were in code: #2564 (a quadratic
      UNIQUE walk), #2571 (an i32 overflow), #2395 (markup injection).
  - About a fifth of PRs are task-only or docs-only: about 57 of 300 had a `tasks:`, `docs` or
    `steward` title. Every PR still ran security, architecture and quality.
  - Each round costs tokens, and usually a push and a CI run.
  - Architecture's and quality's findings overlap, and `scripts/check_duplication.py` already
    enforces part of quality's job.
- **Decision:**
  - **One design reviewer.** `reviewer-design` replaces `reviewer-architecture` and
    `reviewer-quality`. Its mandate is the union of their checks, so every blocking rule of
    either stays. It runs on Sonnet like both did (0015) and runs on every PR.
  - **Security on executable changes.** `reviewer-security` reviews every diff unless every path
    is under `docs/` or `eval/suites/` and none is under `docs/process/`. Unknown or empty path
    lists get every reviewer.
    - Such a diff holds only task files, decisions, specs, research and semantics notes, and
      suite cases, with nothing to execute.
    - Security's other targets keep it: crates, bindings, scripts, tools, CI, `.claude/`,
      `CLAUDE.md`, eval baselines and accuracy lists, and `quality/` (gate weakening).
  - **Excel fidelity is unchanged.**
  - **Task-only diffs first.** A diff of only `docs/tasks/*.md` files needs no reviewer (decision
    0231); `applicable_reviewers` checks that before the rules above.
  - **Tooling.** The one definition of the panel is the `review` skill.
    - `scripts/review_panel.py` mirrors it: `REVIEWERS = (security, design, excel-fidelity)`,
      `SECURITY_SKIP_PATHS`, `SECURITY_KEEP_PATHS`, `applicable_reviewers`, and the legacy fold
      `with_legacy_design` over `LEGACY_DESIGN` (see Transition). `scripts/merge_ready.py`
      re-exports it, and `scripts/test_merge_ready.py` fails when the mirror and the skill differ.
    - `make verdict-record` takes `REVIEWER=security|design|excel-fidelity`.
- **Transition:**
  - Markers and verdict records that name `architecture` or `quality` still parse. A head on which
    both approve counts as approved by `design`; one of them alone, or either with another
    verdict, counts as a `design` REQUEST_CHANGES. A `design` verdict of its own wins over the
    pair.
  - So PRs marked or reviewed before the switch stay ready, and a session still running the old
    skill text keeps working until it reads the new one.
- **Consequences:**
  - A code PR runs two Sonnet reviewers instead of three, plus excel-fidelity where it applies. A
    task-only or docs-only PR runs one.
  - The steward's weekly spot-audit (0015) now also checks for misses by the narrower panel. A
    miss on a docs-only diff that security would have caught widens `SECURITY_SKIP_PATHS`'
    complement through a follow-up decision.
  - The governance rule ("protected paths need design and security approval") still holds:
    every protected path but `docs/process/` lies outside the skip paths, and `docs/process/`
    keeps security explicitly.
