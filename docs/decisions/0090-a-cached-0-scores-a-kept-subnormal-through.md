# 0090 — A cached 0 scores a kept subnormal through a cached override whose probe companion is FALSE

- **Status:** accepted (2026-10-20, T-2210). It extends decision 0042 to excel-cached cells and adds a
  second class of `cached_overrides` line under decision 0032 ("or for a later decision that says so").
  No baseline line is hand-edited and no gate is weakened.
- **Context:**
  - Excel writes a kept subnormal to the saved xlsx as `<v>0</v>` (decision 0089,
    probes/subnormal-saved-as-zero). So ironcalc-calc `MATH_AND_TRIGONOMETRY/EXP_SIGN.xlsx` EXP_SIGN!B44,
    `=EXP(A44)` with A44 = -745 stored, caches 0, while Excel holds 2^-1074 in memory, as the engine does.
    Decision 0089 named the cell an M5 exit exception and filed this task for a scoring rule.
  - Decision 0042's rule (`graph_hides_subnormal`) passes an excel-graph 0 against an engine subnormal only
    when the same spec case has a FALSE companion `=(<formula>)=0`. An xlsx case has no companions, so the
    rule cannot reach a cached cell without evidence from another case.
  - Decision 0032's `cached_overrides` file already does the cross-case part: a line points one scored
    cached cell at a golden oracle cell "computing the same formula on the same inputs", and reviewers
    check that sameness.
- **Options considered:**
  - *(a) A new checked-in evidence list* of folded formulas (`EXP(-745)`), matched against each cached
    cell's formula with its precedents' constants folded in. Mechanical sameness, but a formula folder
    and a new file format in the harness for one cell today. Rejected as not worth its code.
  - *(b) Reuse `cached_overrides`.* The line replaces B44's cached 0 with probes/subnormal-saved-as-zero
    S!A1 (`=EXP(-745)`, excel-graph 0), and the harness reads that probe's spec beside the sidecar: when
    the evidence cell's own companion `=(<its formula>)=0` is FALSE in the same excel-graph sidecar, the
    override carries decision 0042's `from_graph` flag. About 40 lines, no new format. Chosen.
  - *(c) Keep the exit exception* (decision 0089). It records a correct engine as failing. Rejected now
    that (b) is cheap.
- **Decision:**
  - A `cached_overrides` line may name a cached 0 whose Excel value is a kept subnormal, pointing at an
    excel-graph probe cell of the same formula on the same inputs. The harness (`overrides::evidence`)
    sets the expectation's `from_graph` flag only when the probe's spec (the `.case.json` beside the
    sidecar) has a cell `=(<evidence cell's formula>)=0` whose value in that excel-graph sidecar is FALSE.
    Then decision 0042's rule applies unchanged: exactly 0 expected, a non-zero engine value below the
    smallest normal double.
  - Where Excel flushes (`*`, `/`, PRODUCT, ...), the companion is TRUE and the cell still fails, as does
    an override without a companion, with a companion for another formula, from excel-desktop, or whose
    sidecar has no `.case.json` beside it (the flag fails closed).
  - As for every override, the PR adding the line cites the probe, and reviewers check that the probe
    cell computes the same formula on the same inputs (the harness cannot).
- **Consequences:**
  - EXP_SIGN!B44 passes and leaves eval/baseline/ironcalc-calc.txt (`make eval-baseline`); its line
    leaves eval/milestones/exit-exceptions.tsv. `rustsheet_xlsx`'s generated tests skip it, as for every
    override (build.rs).
  - Later cells of this kind take an override line citing this decision instead of an exit exception
    (this replaces decision 0089 item 2 for cells a probe covers).
  - The probe's spec is now load-bearing: editing its companions changes how the overridden cell scores.
