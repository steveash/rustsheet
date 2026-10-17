# 0070 — Gate displayed text with TC and TF baseline lines

- **Status:** accepted (2026-10-16, T-1739)
- **Context:** Decision 0068 scores displayed text (a sidecar's `text` map) as its own verdict and
  keeps it out of the baselines and the gate until a formatted-text suite exists. gen-m10 (T-1611)
  now holds 10,400 oracle text cells, and M10's exit criterion is that suite at 100%. Without a
  gate, a text cell that passes today can start failing unnoticed.
- **Options considered:**
  - *A sibling file per suite* (`eval/baseline/<suite>.text.txt`). It doubles the files the
    governance check and reviewers must watch, for no gain.
  - *Count text cells in the `C` line and list text failures as `F` items.* It changes the meaning
    of `C` (checked value cells) that the milestone lists, the probe-milestone report and the
    evidence floor read.
  - *Two new line kinds in the same file*, read only by the text gate.
- **Decision:** `eval/baseline/<suite>.txt` gains `TC<TAB>case<TAB>text-cells` (the number of
  scored text cells of a case) and `TF<TAB>case<TAB>Sheet!A1` (a known text failure). They are
  written by `update-baseline` and checked by `check` with the value ratchet's rules: a failing
  text cell not listed, in a case whose `TC` count is unchanged, is a regression; a listed cell
  that now passes is a fix; a case whose text count changed or that is new is new coverage. Text
  is gated per cell in every gated suite (`baseline` other than `none`), whatever the suite's value
  mode, and a case with no scored text cells has no `TC` line. `C` and `F` keep their meaning.
- **Consequences:** `make eval-baseline` records gen-m10's failing text cells once (this PR).
  Later formatter fixes must ratchet them with `make eval-baseline`, and a text regression needs
  `--allow-regressions` and a decision record, as a value regression does. The summary line of
  `eval-check` reports text known failures beside value ones. This amends decision 0068's "text
  results never change the baselines or the gate".
