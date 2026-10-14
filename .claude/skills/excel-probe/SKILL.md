---
name: excel-probe
description: How to establish exactly what Microsoft 365 Excel does for a formula or edge case in RustSheet - find existing evidence, write a probe case, get oracle values, and record the finding. Use whenever an implementation depends on Excel behavior you cannot cite.
---
# Establishing Excel behavior

Never implement Excel semantics from memory or from function documentation alone — Excel's
observable behavior (including bugs and undocumented edge cases) is the spec. Excel is always
right: if the answer looks wrong, arbitrary or inconsistent, record it and match it anyway.

1. **Search existing evidence**
   - `docs/excel-semantics/` (curated findings), `rg -n "<FUNC>\(" eval/suites` (cases),
     `make eval-show CASE=...` on cases that already exercise it.
   - IronCalc's Excel-authored workbooks (`crates/rustsheet-xlsx/tests/calc_tests/`) carry Excel's
     cached values: `./target/release/rustsheet-eval snapshot <file.xlsx>`.
2. **Write a probe case** in `eval/suites/probes/cases/<topic>.case.json` (`rustsheet-case/v1`):
   one cell per question, minimal inputs, `"expected": {}`. Group related questions in one file.
   Give it exactly one milestone tag: the milestone of the function or feature it probes (the
   highest catalog milestone, `eval/catalog/scope.json`, of the subject functions; 3-D and external
   references and defined-name bodies are M8, number-format codes M10), `M2` for an M0-M2
   subject, never the milestone of the task that wrote it (decisions 0051, 0057, 0058). A case
   that mixes an M2 subject with later ones is tagged `M2`; give its later-subject cells cell lines
   in `eval/milestones/probe-milestones.tsv` (by decision record), or split the case.
3. **Get oracle values**
   - If an Excel backend is available (`cd tools && uv run rustsheet-oracle status`):
     `uv run rustsheet-oracle fill -b excel-graph --suite probes` → commits `*.oracle.json`.
   - Otherwise leave the case pending (it shows as "pending" in reports), mark the task
     `blocked` on oracle with a link to the case, and pick other work. The scheduled oracle
     workflow (`.github/workflows/oracle.yml`) fills pending cases.
   - If only desktop Excel can answer (excel-graph cannot evaluate it, or contradicts a desktop
     cached value), do not wait for it: implement on excel-graph evidence where it exists, and
     put the desktop question in M14, Phase 4 (decision 0039).
4. **Record the finding** in `docs/excel-semantics/<area>.md` (the canonical quirk list): the
   rule in one sentence, the probe case id as evidence, and any surprising consequences. Record
   every finding, including ones that look like Excel bugs and ones no task implements yet.
   Cite it in code comments where the quirk is implemented (`// Excel: see docs/excel-semantics/rounding.md#round-half`).
