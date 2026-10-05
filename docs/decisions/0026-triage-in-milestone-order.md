# 0026 — Triage and pick functions work in milestone order

- **Status:** accepted (2026-10-05; T-0018)
- **Context:** on 2026-10-04, 12 of the functions lane's open tasks were M9 numeric tails (two
  attempts already blocked: T-0901, T-0925) while M2's everyday functions had no oracle evidence
  yet. The gap-triager ranked groups only by failing cells × frequency prior, so large
  last-digit tails outranked current-milestone work, and fitting tasks were attempted with only
  the few values cached in Excel-authored workbooks.
- **Decision:**
  - The gap-triager (`.claude/agents/gap-triager.md`) ranks groups by the function's catalog
    milestone (earliest first), then failing cells × prior, and sets each task's milestone from
    the catalog.
  - `scripts/tasks.py next` keeps its existing order (P0, then handoffs, then milestone, then
    priority; decision 0023), so M9 tasks are picked only when nothing earlier is ready in the
    lane. A test pins that order.
  - A numeric-fitting task that needs more Excel points than the cached workbooks give says
    "Needs an oracle sweep" and depends on a research-lane sweep task (T-0942 for the current
    M9 set: T-0901, T-0908, T-0915, T-0916, T-0917, T-0925).
- **Consequences:**
  - Those six M9 tasks leave the ready queue until T-0942 lands its sweeps.
  - The opus functions run spends its time on earlier milestones first.
