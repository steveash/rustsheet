# 0037 — Allow two gen-m2 regressions unmasked by T-0212

- **Status:** accepted (2026-10-06, T-0212)
- **Context:** T-0212 replaces the approximate-match search of VLOOKUP/HLOOKUP/LOOKUP/MATCH with
  Excel's type-skipping binary search (docs/excel-semantics/lookup.md "Approximate match";
  probes/lookup-approximate-search, excel-graph). That fixes 420 gen-m2 cells and 1 poi cell. Two
  other cells passed before only by accident, because the old search compared across types and
  returned a value where the correct search returns `#N/A` or a different row, and a second,
  unrelated bug then happened to give Excel's answer:
  - `gen-m2/2cfe279fe77a000d` Calc!A328 and `gen-m2/ef5c55a81783b7d5` Calc!A151: the type-skipping
    search now probes the only text cell of the column, and compares it in code-point order. Excel
    ranks "ü" below "x" and "?b" below "1" (collation, T-0220), so Excel finds that row and we do not.
  (A third, `gen-m2/1b6ceed7489e0800` Calc!A332, was unmasked too while this branch was open,
  through `&` not broadcasting a scalar error; T-0129 fixed that on `main` first, so it passes.)
- **Options considered:**
  - *Keep the old search for these shapes.* It contradicts the oracle on 420 cells and the probe.
  - *Fix collation in the same PR.* T-0220 is cross-cutting and needs its own probe and spec; it
    would make this PR two tasks (CLAUDE.md directive 4).
  - *Record the two as known failures and give them to T-0220, which owns their root cause.*
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-0212 PR, for
  exactly these two cells. They are added to T-0220's acceptance list.
- **Consequences:** `eval/baseline/gen-m2.txt` loses 420 lines and gains these two. Reviewers
  confirm the baseline diff adds no other line. T-0220 must make them pass.
