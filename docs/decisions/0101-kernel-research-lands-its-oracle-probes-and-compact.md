# 0101 — Kernel research lands its oracle probes and compact data in git, about 40 MB for T-2464

- **Status:** accepted (2026-10-23, T-2464)
- **Context:**
  - T-2464 cracked most of Excel's incomplete beta in two rounds of research workers. Its evidence is
    about 40 MB of tracked files: 20 MB of oracle-filled probe cases in
    `eval/suites/probes-numeric/cases/t2464-*` (sidecars from excel-graph, 2026-10-23) and 20 MB in
    `docs/research/t2464/` (scripts, RESULTS.md files, the extracted cell tables `cells.tsv` and
    `beta_exact.tsv`, probe meta files and the oracle outputs of probes that never became eval cases).
    The next largest research directory is under 1 MB.
  - The architecture reviewer asked for either a trim or a record. The oracle values cannot be
    regenerated offline: they cost oracle time, and Excel for the web changes (T-2337). Without them
    the statistics.md entries and the engine tasks (T-2502..T-2513, T-2633) could not be re-checked.
  - Caches, pickles, copies of other workers' code and tables the scripts rebuild in seconds are
    already git-ignored (`docs/research/t2464/.gitignore`), and so are the 87 exploratory scripts no
    RESULTS.md cites.
- **Decision:** A kernel-research task may commit its oracle evidence (probe cases with sidecars, or the
  oracle outputs of probes kept in its research directory) and the compact tables its RESULTS.md
  files are scored on, up to about 50 MB per task. Anything a script rebuilds from those files is
  git-ignored. More than that needs its own decision record.
- **Consequences:**
  - Clones grow by about 40 MB once: T-2464 is squash-merged, so main carries only its final tree.
  - The probes-numeric cases are ungated (`gate = false`) and add about 4 s to `make eval` (18 cases, 167,838 cells).
  - When the engine tasks land (T-2505 and its successors), the follow-up research task (T-2512 or
    T-2513) may delete outputs that only intermediate scripts read, keeping the eval cases.
