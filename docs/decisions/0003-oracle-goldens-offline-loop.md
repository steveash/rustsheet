# 0003 — Excel results are committed goldens; the agent loop runs offline

- **Status:** accepted (2026-10-01)
- **Context:** Real Excel is slow, rate-limited and needs credentials (the Graph API needs a
  work/school tenant; desktop needs Windows). Agents must iterate fast and reproducibly.
- **Decision:**
  - Oracle runs are batched, keyed by content hash, and committed as `*.oracle.json` sidecars.
  - Excel-authored xlsx cached values count as oracle data.
  - Every backend returns a recalculated xlsx parsed by the single Rust snapshot parser.
  - LibreOffice is triage-only and never golden.
- **Consequences:** Agents never call Excel in the inner loop. Pending cases are visible in reports
  and are filled by the oracle workflow. Each golden records the Excel build.
