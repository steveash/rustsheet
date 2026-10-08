---
id: T-0289
title: "spec2xlsx writes numbers losslessly (17 digits) so Excel and the engine read the same double"
milestone: M2
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A number in a spec case reaches Excel as the exact double the case file holds, so the harness no
longer rounds spec numbers to 16 digits for the engine (`as_written` in
crates/rustsheet-eval/src/engine.rs goes away).

## Context
T-1156 found that tools/src/rustsheet_tools/spec2xlsx.py writes numbers through XlsxWriter, which
stores `{number:.16G}` (16 significant digits). A spec value that needs 17 digits, such as 2^1000
(1.0715086071862673E+301), reached Excel three ulps low, and made probes/geomean-exponent-sum-2
C12-C18 `#NUM!`. T-1156 made the engine read the same 16 digits (`as_written`). Excel's own files store
17 digits, so a lossless writer is closer to real workbooks. Ways to do it: patch the `<v>` text after
writing, as spec2xlsx already does for error cells, or write numbers with `repr`.
- Check how `rustsheet-oracle` hashes oracle inputs. If the hash covers the xlsx bytes, every case
  with a 17-digit number changes input and must be refilled from excel-graph.
- Cases that change meaning: geomean-exponent-sum-2 B11-B18 (C11-C18 become exact roots, like
  geomean-pow2-count E). `rg` the cases for numbers with 17 significant digits to list the rest.

## Acceptance criteria
- [x] spec2xlsx writes every number so that it parses back to the same double, with a test in tools/tests.
- [x] `as_written` is removed, and eval-harness.md "Input conventions" is updated.
- [x] Affected oracle sidecars are refilled from excel-graph, numbers.md cites the new values, and there is no regression.
- [x] `make check` green; reviewers approve.

## Out of scope
- Typed entry (`Entered`) cells, which go through Excel's own parser.

## Outcome
spec2xlsx now rewrites the `<v>` of any number whose 16-digit text is not the same double to Python's shortest
round-trip text (`_store_numbers`), tested in tools/tests/test_spec2xlsx_numbers.py. `as_written` and its test are gone
from crates/rustsheet-eval/src/engine.rs; eval-harness.md "Input conventions" and numbers.md (GEOMEAN) are updated.
Refilled from excel-graph: probes/geomean-pow2-count and geomean-exponent-sum-2 (the only cases holding 17-digit
numbers); a stored 2^1000 now equals `2^1000` (Q3, Q4 are 0) and the `#NUM!` cells are exact roots. Eval: probes known
failures 353 -> 350 (fixed 3), regressions 0.
