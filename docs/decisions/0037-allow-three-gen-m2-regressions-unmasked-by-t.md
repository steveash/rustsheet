# 0037 — Allow three gen-m2 regressions unmasked by T-0212

- **Status:** accepted (2026-10-06, T-0212)
- **Context:** T-0212 replaces the approximate-match search of VLOOKUP/HLOOKUP/LOOKUP/MATCH with
  Excel's type-skipping binary search (docs/excel-semantics/lookup.md "Approximate match";
  probes/lookup-approximate-search, excel-graph). That fixes 419 gen-m2 cells and 1 poi cell. Three
  other cells passed before only by accident, because the old search compared across types and
  returned a value where the correct search returns `#N/A` or a different row, and a second,
  unrelated bug then happened to give Excel's answer:
  - `gen-m2/2cfe279fe77a000d` Calc!A328 and `gen-m2/ef5c55a81783b7d5` Calc!A151: the type-skipping
    search now probes the only text cell of the column, and compares it in code-point order. Excel
    ranks "ü" below "x" and "?b" below "1" (collation, T-0220), so Excel finds that row and we do not.
  - `gen-m2/1b6ceed7489e0800` Calc!A332: VLOOKUP is now correctly `#N/A` (probe N17 shape: no text
    below the key), and `{#VALUE!,#VALUE!}&#N/A` returns the right operand's scalar error instead of
    broadcasting the leftmost one (T-0263).
- **Options considered:**
  - *Keep the old search for these shapes.* It contradicts the oracle on 419 cells and the probe.
  - *Fix collation and `&` in the same PR.* T-0220 is cross-cutting and needs its own probe and
    spec; T-0263 is a separate operator fix. Either would make this PR two tasks (CLAUDE.md directive 4).
  - *Record the three as known failures and give them to the tasks that own their root causes.*
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-0212 PR, for
  exactly these three cells. A328 and A151 are added to T-0220's acceptance list, A332 to T-0263's.
- **Consequences:** `eval/baseline/gen-m2.txt` loses 419 lines and gains these three. Reviewers
  confirm the baseline diff adds no other line. T-0220 and T-0263 must make them pass.
