# 0068 — Displayed-text expectations in spec cases and oracle sidecars

- **Status:** accepted (2026-10-16, T-1610)
- **Context:** M10's exit criterion is a formatted-text oracle suite (≥ 10k cells) at 100%. An xlsx
  stores the raw value, not the text Excel shows, so a sidecar built from the saved file cannot
  carry it. Graph's range resource returns the shown text (`text`) of a recalculated workbook.
  probes/displayed-text-formats shows that excel-graph's `text` does not depend on the column
  width (`12345678` at width 4 stays "12345678"; an invalid date is 255 `#`).
- **Decision:**
  - A spec case may list cells in a top-level `"text": ["Sheet!A1", ...]`. spec2xlsx records them
    as `rustsheet.text.<n>` custom properties, and the excel-graph backend reads each cell's
    `Range.text` after `calculate`, in the same session. The fill writes them to the sidecar's
    optional `text` map (sheet → A1 → text). A sheet may pin `"column_widths"`; the engine adapter
    ignores it, since excel-graph's text does not depend on the width.
  - The eval compares the sidecar text with `rustsheet::Workbook::formatted` as a separate verdict
    (`text_expect.rs`). Text results never change value scoring, a case's status, the baselines or
    the gate. The report shows a text pass rate per suite and lists every failing text cell.
  - Other backends write no text, so their listed cells stay pending.
- **Consequences:** M10's generated suite (T-1611) can carry text expectations. Gating text
  (a baseline of failing text cells) is a later decision, to be made once the suite exists. A
  desktop `.Text` (width-dependent `####`) would need its own field. Engine mismatches found by
  the first probe are filed as T-1711.
